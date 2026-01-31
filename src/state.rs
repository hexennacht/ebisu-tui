use crate::models::{Category, CategoryBalance};

/// Input mode for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
}

/// Active input field identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveInput {
    #[default]
    None,
    Amount,
    Description,
    Category,
}

/// Active tab/page
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveTab {
    #[default]
    Dashboard,
    AddFunds,
    AddExpense,
    Reports,
    Settings,
}

impl ActiveTab {
    pub fn next(&self) -> Self {
        match self {
            Self::Dashboard => Self::AddFunds,
            Self::AddFunds => Self::AddExpense,
            Self::AddExpense => Self::Reports,
            Self::Reports => Self::Settings,
            Self::Settings => Self::Dashboard,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Dashboard => Self::Settings,
            Self::AddFunds => Self::Dashboard,
            Self::AddExpense => Self::AddFunds,
            Self::Reports => Self::AddExpense,
            Self::Settings => Self::Reports,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::AddFunds => "Add Funds",
            Self::AddExpense => "Add Expense",
            Self::Reports => "Reports",
            Self::Settings => "Settings",
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::Dashboard,
            Self::AddFunds,
            Self::AddExpense,
            Self::Reports,
            Self::Settings,
        ]
    }
}

/// Shared application state
#[derive(Debug, Default)]
pub struct State {
    /// Cached category configurations
    pub categories: Vec<Category>,
    /// Cached current balances
    pub balances: Vec<CategoryBalance>,
    /// Current input mode
    pub input_mode: InputMode,
    /// Which input field is active
    pub active_input: ActiveInput,
    /// Current active tab
    pub active_tab: ActiveTab,
    /// Selected category index (for forms)
    pub selected_category: usize,
    /// Input buffer for amount
    pub amount_input: String,
    /// Input buffer for description
    pub description_input: String,
    /// Status message to display
    pub status_message: Option<String>,
    /// Whether to show help overlay
    pub show_help: bool,

    // Reporting state
    pub report_date_range: crate::models::DateRange,
    pub transactions: Vec<crate::models::Transaction>,
    pub summary_stats: Option<crate::models::SummaryStats>,
}

impl State {
    pub fn new() -> Self {
        Self {
            report_date_range: crate::models::DateRange::Month,
            ..Default::default()
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    pub fn clear_inputs(&mut self) {
        self.amount_input.clear();
        self.description_input.clear();
        self.selected_category = 0;
        self.active_input = ActiveInput::None;
        self.input_mode = InputMode::Normal;
    }
}
