use chrono::{DateTime, Local};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Category {
    pub id: i64,
    pub name: CategoryName,
    pub limit_percentage: Decimal,
    pub overflow_to_id: Option<i64>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Display,
    EnumString,
    EnumIter,
)]
pub enum CategoryName {
    Needs,
    Wants,
    Culture,
    Unexpected,
    Savings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fund {
    pub id: i64,
    pub amount: Decimal,
    pub added_at: DateTime<Local>,
    pub remaining_balance_rolled: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i64,
    pub category_id: i64,
    pub amount: Decimal,
    pub description: Option<String>,
    pub created_at: DateTime<Local>,
    pub overflow_from_id: Option<i64>,
    // Enriched data (joined)
    pub category_name: Option<CategoryName>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryBalance {
    pub category_id: i64,
    pub category_name: CategoryName,
    pub available: Decimal,
    pub allocated: Decimal,
    pub spent: Decimal,
    pub last_updated: DateTime<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryStats {
    pub total_funds_added: Decimal,
    pub total_spent: Decimal,
    pub current_settings: Vec<Category>,
    pub balances: Vec<CategoryBalance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, EnumIter)]
pub enum DateRange {
    Today,
    Last7Days,
    Month,
    Year,
    FiveYears,
}
