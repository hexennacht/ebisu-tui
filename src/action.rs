use crate::models::DateRange;

/// Application actions representing all possible state transitions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    // Core events
    Tick,
    Render,
    Resize(u16, u16),
    Quit,

    // Navigation
    NextTab,
    PrevTab,
    Up,
    Down,
    Left,
    Right,

    // Input modes
    EnterInsert,
    EnterNormal,

    // Form actions
    SubmitTransaction,
    SubmitFunds,
    CancelInput,

    // Data refresh
    RefreshBalances,
    RefreshCategories,

    // Reporting
    ChangeDateRange(DateRange),

    // UI toggles
    ToggleHelp,

    // Text input
    InputChar(char),
    InputBackspace,
    InputDelete,

    // Category selection
    SelectCategory(usize),

    // Settings
    StartEditingCategory,
    ConfirmCategoryEdit, // Renamed from SaveCategoryLimit to be clearer about memory update
    SaveSettings,        // Triggers DB persist
}

/// Direction for navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
