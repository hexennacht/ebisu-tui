# Ebisu TUI - Development Knowledge & Learnings

This document captures the key architectural decisions, database design patterns, business logic implementations, and technical learnings gained during the development of Ebisu TUI.

## 🏗️ Architecture Overview

The application follows a standard TUI architecture powered by `ratatui` and `crossterm`.

*   **State Management**: Centralized `State` struct in `src/state.rs` holds all application data (categories, balances, input buffers, views).
*   **Action Dispatch**: User inputs are converted into `Action` enums in `src/action.rs` (e.g., `SubmitFunds`, `SaveSettings`), which are processed by the `update` loop in `src/app.rs`.
*   **Database Layer**: All SQLite interactions are encapsulated in `src/database.rs`, exposing high-level async methods (e.g., `add_funds`, `get_transactions`).
*   **UI Rendering**: Stateles logic in `draw_*` methods in `src/app.rs` renders the UI based on the current `State` each frame.

## 💾 Database Design (SQLite)

### Schema Highlights
*   **`categories`**: Configures spending limits (`limit_percentage`) and overflow chains (`overflow_to_id`).
*   **`funds`**: Tracks income. Columns: `amount`, `added_at`, `remaining_balance_rolled`.
*   **`category_balances`**: Stateful table tracking `available`, `allocated`, and `spent` for each category.
*   **`transactions`**: explicit `created_at` timestamp in **RFC3339** format.

### Key Learnings & Date Handling
> [!IMPORTANT]
> **Date Format Consistency is Critical**
> SQLite's default `CURRENT_TIMESTAMP` stores UTC dates as "YYYY-MM-DD HH:MM:SS".
> Rust's `chrono::Local::now()` uses RFC3339 ("YYYY-MM-DDTHH:MM:SS+Offset").
> **Solution**: We explicitly insert dates using `Local::now().to_rfc3339()` in Rust rather than relying on SQLite defaults. This ensures correct sorting and filtering for "Today" reports in the local time zone.

## 🧠 Core Business Logic (Kakeibo Method)

### Fund Allocation
*   **Trigger**: User adds funds (e.g., salary).
*   **Calculations**:
    1.  Calculates **Rollover**: Any positive `available - spent` from non-savings categories is summed up.
    2.  **Allocation**: New funds are distributed based on `category.limit_percentage`.
    3.  **Savings**: Receiving category for rollovers. `allocated` for Savings = (Fund * %) + Total Rollover.
*   **Reset**: `spent` is reset to 0 for all categories upon new fund addition.

### Overflow Logic
*   **Chain**: Configured via `overflow_to_id`. Standard chain:
    `Specific Category` → `Unexpected` → `Savings`.
*   **Behavior**: If a category has insufficient funds, the system recursively checks the next category in the chain to cover the difference. Transactions record the original category, but balances are deducted from the overflow source.

## 🎨 UI/UX Patterns

### Batch Saving Strategy
*   **Context**: Settings (Category Limits).
*   **Pattern**:
    *   **Edit**: Changes (`Enter`/`i`) update the **in-memory** state immediately. This allows the "Total Allocation" indicator to update in real-time.
    *   **Persist**: A "Save Changes" button at the bottom of the list triggers the DB write for *all* categories at once.
    *   **Benefit**: Prevents partial/inconsistent states in the DB and provides a "Confirm" step for the user.

### Input Modes
*   **Normal Mode**: Navigation (`j`/`k`), Tab switching (`Tab`), Interaction (`Enter`).
*   **Insert Mode**: Text input for amounts/descriptions. `Esc` returns to Normal.

## 🔧 Technical Specifics

*   **Concurrency**: `tokio` runtime drives the main loop and async DB calls.
*   **Decimal Handling**: `rust_decimal` used for all monetary values to avoid floating point errors. IDs should be formatted appropriately (e.g., IDR currency formatting).
*   **Error Handling**: `anyhow` for app-level errors, ensuring robust crash reporting (though we aim to handle errors gracefully in the UI via status messages).
