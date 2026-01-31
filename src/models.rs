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

impl Default for DateRange {
    fn default() -> Self {
        Self::Month
    }
}

impl DateRange {
    pub fn get_dates(&self) -> (DateTime<Local>, DateTime<Local>) {
        let now = Local::now();
        let end_of_day = now
            .date_naive()
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_local_timezone(Local)
            .unwrap();

        let start_date = match self {
            DateRange::Today => now
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_local_timezone(Local)
                .unwrap(),
            DateRange::Last7Days => (now - chrono::Duration::days(7)),
            DateRange::Month => (now - chrono::Duration::days(30)),
            DateRange::Year => (now - chrono::Duration::days(365)),
            DateRange::FiveYears => (now - chrono::Duration::days(365 * 5)),
        };

        (start_date, end_of_day)
    }

    pub fn title(&self) -> &str {
        match self {
            DateRange::Today => "Today",
            DateRange::Last7Days => "Last 7 Days",
            DateRange::Month => "Last 30 Days",
            DateRange::Year => "Last Year",
            DateRange::FiveYears => "Last 5 Years",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            DateRange::Today => DateRange::Last7Days,
            DateRange::Last7Days => DateRange::Month,
            DateRange::Month => DateRange::Year,
            DateRange::Year => DateRange::FiveYears,
            DateRange::FiveYears => DateRange::Today,
        }
    }
}
