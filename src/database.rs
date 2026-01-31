use crate::error::{AppError, Result};
use crate::models::{Category, CategoryBalance, CategoryName};
use chrono::Local;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use std::str::FromStr;

#[derive(Debug)]
pub struct DB {
    conn: libsql::Connection,
}

impl DB {
    #[allow(dead_code)]
    pub async fn new() -> Result<Self> {
        let db = libsql::Builder::new_local("ebisu.db").build().await?;
        let conn = db.connect()?;

        let db_instance = Self { conn };
        db_instance.initialize_schema().await?;

        Ok(db_instance)
    }

    async fn initialize_schema(&self) -> Result<()> {
        let schema = include_str!("../schema.sql");
        self.conn.execute_batch(schema).await?;
        self.seed_categories().await?;
        Ok(())
    }

    async fn seed_categories(&self) -> Result<()> {
        let count: i64 = self
            .conn
            .query("SELECT COUNT(*) FROM categories", ())
            .await?
            .next()
            .await?
            .ok_or(AppError::Database(libsql::Error::QueryReturnedNoRows))?
            .get(0)?;

        if count == 0 {
            // Needs -> Unexpected -> Savings
            // Wants -> Unexpected -> Savings
            // Culture -> Unexpected -> Savings
            // Unexpected -> Savings
            // Savings

            let savings_id = self
                .insert_category(
                    CategoryName::Savings,
                    Decimal::from_f64(50.0).unwrap(),
                    None,
                )
                .await?;
            let unexpected_id = self
                .insert_category(
                    CategoryName::Unexpected,
                    Decimal::from_f64(10.0).unwrap(),
                    Some(savings_id),
                )
                .await?;

            self.insert_category(
                CategoryName::Needs,
                Decimal::from_f64(30.0).unwrap(),
                Some(unexpected_id),
            )
            .await?;
            self.insert_category(
                CategoryName::Wants,
                Decimal::from_f64(5.0).unwrap(),
                Some(unexpected_id),
            )
            .await?;
            self.insert_category(
                CategoryName::Culture,
                Decimal::from_f64(5.0).unwrap(),
                Some(unexpected_id),
            )
            .await?;
        }
        Ok(())
    }

    async fn insert_category(
        &self,
        name: CategoryName,
        limit: Decimal,
        overflow_to: Option<i64>,
    ) -> Result<i64> {
        let (sql, params): (&str, Vec<String>) = if let Some(oid) = overflow_to {
            (
                "INSERT INTO categories (name, limit_percentage, overflow_to_id) VALUES (?, ?, ?) RETURNING id",
                vec![name.to_string(), limit.to_string(), oid.to_string()],
            )
        } else {
            (
                "INSERT INTO categories (name, limit_percentage, overflow_to_id) VALUES (?, ?, NULL) RETURNING id",
                vec![name.to_string(), limit.to_string()],
            )
        };

        let mut stmt = self.conn.prepare(sql).await?;
        let mut rows = stmt.query(params).await?;

        let row = rows
            .next()
            .await?
            .ok_or(AppError::Database(libsql::Error::QueryReturnedNoRows))?;
        let id: i64 = row.get(0)?;

        // Also init balance
        let _ = self
            .conn
            .execute(
                "INSERT OR IGNORE INTO category_balances (category_id) VALUES (?)",
                [id],
            )
            .await?;

        Ok(id)
    }

    pub async fn get_categories(&self) -> Result<Vec<Category>> {
        let mut rows = self
            .conn
            .query(
                "SELECT id, name, limit_percentage, overflow_to_id FROM categories",
                (),
            )
            .await?;
        let mut categories = Vec::new();

        while let Some(row) = rows.next().await? {
            let cat_row: CategoryRow =
                libsql::de::from_row(&row).map_err(|e| AppError::Deserialization(e.to_string()))?;

            categories.push(Category {
                id: cat_row.id,
                name: CategoryName::from_str(&cat_row.name).map_err(|_| {
                    AppError::Validation(format!("Invalid category name: {}", cat_row.name))
                })?,
                limit_percentage: cat_row.limit_percentage,
                overflow_to_id: cat_row.overflow_to_id,
            });
        }
        Ok(categories)
    }

    // FUND ALLOCATION LOGIC
    #[allow(dead_code)]
    pub async fn add_funds(&self, amount: Decimal) -> Result<()> {
        let balances = self.get_category_balances().await?;

        let mut total_rollover = Decimal::ZERO;
        for bal in &balances {
            if bal.category_name != CategoryName::Savings {
                let remaining = bal.allocated - bal.spent;
                if remaining > Decimal::ZERO {
                    total_rollover += remaining;
                }
            }
        }

        // Use transaction for atomic updates
        let tx = self
            .conn
            .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
            .await?;

        tx.execute(
            "INSERT INTO funds (amount, remaining_balance_rolled) VALUES (?, ?)",
            [amount.to_string(), total_rollover.to_string()],
        )
        .await?;

        let categories = self.get_categories().await?;
        for cat in categories {
            let portion = amount * (cat.limit_percentage / Decimal::from_i32(100).unwrap());
            let mut new_allocation = portion;
            let mut new_available = portion;

            if cat.name == CategoryName::Savings {
                new_allocation += total_rollover;
                let current_savings_bal = balances
                    .iter()
                    .find(|b| b.category_name == CategoryName::Savings);
                let previous_savings = if let Some(b) = current_savings_bal {
                    b.available - b.spent
                } else {
                    Decimal::ZERO
                };
                new_available = previous_savings + new_allocation;
            }

            tx.execute(
                "UPDATE category_balances SET available = ?, allocated = ?, spent = '0', last_updated = CURRENT_TIMESTAMP WHERE category_id = ?",
                [new_available.to_string(), new_allocation.to_string(), cat.id.to_string()]
            ).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_category_by_name(&self, name: CategoryName) -> Result<Category> {
        let cats = self.get_categories().await?;
        cats.into_iter()
            .find(|c| c.name == name)
            .ok_or(AppError::CategoryNotFound(name.to_string()))
    }

    #[allow(dead_code)]
    pub async fn get_category_balances(&self) -> Result<Vec<CategoryBalance>> {
        let rows = self
            .conn
            .query(
                r#"SELECT cb.category_id, c.name as category_name, cb.available, cb.allocated, cb.spent, cb.last_updated 
                FROM category_balances cb 
                JOIN categories c ON cb.category_id = c.id"#,
                (),
            )
            .await?;

        let mut balances = Vec::new();
        let mut rows = rows;

        while let Some(row) = rows.next().await? {
            let bal_row: BalanceRow =
                libsql::de::from_row(&row).map_err(|e| AppError::Deserialization(e.to_string()))?;

            balances.push(CategoryBalance {
                category_id: bal_row.category_id,
                category_name: CategoryName::from_str(&bal_row.category_name)
                    .unwrap_or(CategoryName::Unexpected),
                available: bal_row.available,
                allocated: bal_row.allocated,
                spent: bal_row.spent,
                last_updated: Local::now(),
            });
        }
        Ok(balances)
    }

    #[allow(dead_code)]
    pub async fn create_transaction(
        &self,
        category_name: CategoryName,
        amount: Decimal,
        description: Option<String>,
    ) -> Result<()> {
        let categories = self.get_categories().await?;
        let target_cat = categories
            .iter()
            .find(|c| c.name == category_name)
            .ok_or(AppError::CategoryNotFound(category_name.to_string()))?;

        let mut balances = self.get_category_balances().await?;

        let mut remaining_amount_to_cover = amount;
        let mut current_cat_id = target_cat.id;
        let mut updates: Vec<(i64, Decimal)> = Vec::new();

        loop {
            let cat_bal = balances
                .iter_mut()
                .find(|b| b.category_id == current_cat_id)
                .ok_or(AppError::Config("Balance sync error".into()))?;
            let current_remaining = cat_bal.available - cat_bal.spent;

            if current_remaining >= remaining_amount_to_cover {
                updates.push((current_cat_id, remaining_amount_to_cover));
                remaining_amount_to_cover = Decimal::ZERO;
                break;
            } else {
                if current_remaining > Decimal::ZERO {
                    updates.push((current_cat_id, current_remaining));
                    remaining_amount_to_cover -= current_remaining;
                }

                let current_cat_config =
                    categories.iter().find(|c| c.id == current_cat_id).unwrap();
                if let Some(next_id) = current_cat_config.overflow_to_id {
                    current_cat_id = next_id;
                } else {
                    return Err(AppError::InsufficientFunds);
                }
            }
        }

        // Use transaction for atomic writes
        let tx = self
            .conn
            .transaction_with_behavior(libsql::TransactionBehavior::Immediate)
            .await?;

        tx.execute(
            "INSERT INTO transactions (category_id, amount, description, overflow_from_id) VALUES (?, ?, ?, NULL)",
            [target_cat.id.to_string(), amount.to_string(), description.unwrap_or_default()]
        ).await?;

        for (cat_id, deducted) in updates {
            tx.execute(
                "UPDATE category_balances SET spent = spent + ? WHERE category_id = ?",
                [deducted.to_string(), cat_id.to_string()],
            )
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct CategoryRow {
    id: i64,
    name: String,
    limit_percentage: Decimal,
    overflow_to_id: Option<i64>,
}

#[derive(serde::Deserialize)]
struct BalanceRow {
    category_id: i64,
    category_name: String,
    available: Decimal,
    allocated: Decimal,
    spent: Decimal,
    // Include last_updated to match query columns, even if unused or we use String
    #[allow(dead_code)]
    last_updated: String,
}
