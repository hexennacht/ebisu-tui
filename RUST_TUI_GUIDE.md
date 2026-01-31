# Rust TUI Development Guide

This guide serves as a handbook for developing Terminal User Interface (TUI) applications in Rust, based on the technology stack used in the **Ebisu TUI** project. It covers core concepts, dependency usage, and architectural patterns.

---

## 📚 1. Core Rust Concepts for TUI

### Async/Await with Tokio
Modern TUIs often need to perform IO (Database, Network) without freezing the UI.
*   **The Runtime**: We use `tokio` with the `full` feature set.
*   **The Main Loop**: The application runs in a loop that:
    1.  **Polls** for events (Key presses, Ticks).
    2.  **Updates** state based on events.
    3.  **Draws** the UI to the terminal.
*   **Blocking**: ⚠️ NEVER block the main thread. Database calls should be `.await`ed.

### Results & Error Handling
*   **`anyhow::Result`**: Used in the application layer (`main.rs`, `app.rs`) for easy error propagation.
*   **`thiserror`**: Used in library/module layers to define specific, typed errors.
*   **`color_eyre`**: Provides beautiful panic handlers, crucial for TUIs to restore the terminal state before crashing (otherwise the terminal remains broken).

---

## 🛠️ 2. The Dependency Stack

### 🖥️ UI & Rendering: `ratatui`
The core framework for drawing.
*   **Widgets**: Building blocks like `Block`, `List`, `Paragraph`, `Table`.
*   **Layout**: The flexbox-like system using `Layout`, `Constraint`, `Direction`.
    ```rust
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)])
        .split(area);
    ```
*   **Stateful Widgets**: Some widgets (like `List`, `Table`) need a separate `State` object to track selection/scroll (e.g., `ListState`).

### ⌨️ Terminal Control: `crossterm`
Handles the low-level interaction.
*   **Raw Mode**: Disables line buffering (so we get key presses immediately) and echoing.
*   **Events**: `event::read()` captures Key, Mouse, and Resize events.
*   **Alternate Screen**: Switches to a secondary buffer so exiting the app restores the user's shell history.

### 💾 Data Persistence: `libsql` (SQLite)
Embedded SQL database.
*   **Async Interface**: Unlike `rusqlite`, `libsql` plays well with `tokio`.
*   **Parameters**: Always use `?` placeholders to prevent SQL injection.
*   **Transactions**: Use `conn.transaction()` for atomic updates (e.g., deducting funds and creating a transaction record simultaneously).

### 💰 Financial Math: `rust_decimal`
*   **Why**: Floating point (`f64`) is imprecise (0.1 + 0.2 != 0.3).
*   **Usage**: All monetary values use `Decimal`.
*   **Serialization**: Stores transparently as strings/numbers depending on DB config.

### 📅 Time: `chrono`
*   **Standard**: Use `Rich Formatting` timestamp strings (RFC3339) for serialization.
*   **Timezones**: `Local::now()` vs `Utc::now()`. Be consistent.

---

## 🏗️ 3. The TUI Architecture Pattern

We follow the **Elm Architecture** (Model-View-Update):

### 1. The Model (`State`)
Located in `src/state.rs`.
*   A single struct containing *all* application data.
*   UI components read from this, never store their own data (except transient UI state like scroll position).

### 2. The Message (`Action`)
Located in `src/action.rs`.
*   An `enum` representing every possible thing that can happen.
    *   `Quit`
    *   `NavigateUp`
    *   `SaveSettings`
*   This decouples input handling from logic. Key presses just emit Actions.

### 3. The Update (`App::update`)
Located in `src/app.rs`.
*   A giant `match` statement that takes `&mut self` and an `Action`.
*   Mutates the `State` or calls `Database` methods.
*   **Side Effects**: Database calls happen here.

### 4. The View (`draw_*`)
Located in `src/app.rs`.
*   A pure function `(State) -> UI`.
*   Draws the current state of the world using `ratatui` widgets.

---

## 🚀 4. Developer Workflow

### Adding a New Feature
1.  **Define Data**: Add fields to `State` (e.g., `pub show_popup: bool`).
2.  **Define Action**: Add variant to `Action` (e.g., `TogglePopup`).
3.  **Handle Input**: Map key code to Action in `handle_events`.
4.  **Implement Logic**: Handle Action in `update` loop.
5.  **Render**: Add drawing logic in `draw` based on State.

### Common Pitfalls
*   **Terminal Cleanups**: Always ensure `restore_terminal()` is called on panic/exit. Use `color_eyre` hook or `Drop` guards.
*   **Event Blocking**: Don't put `thread::sleep` in the update loop. It stops the UI from redrawing.
*   **Layout Panics**: `Constraint::Percentage` must sum to 100? No, but maintain awareness of 0-height constraints causing widget panics.

---

---

## 🎓 5. Tutorial: Building Your First Ratatui App

This section walks you through the minimal setup to get a "Hello World" TUI running.

### Step 1: Initialize Project
```bash
cargo new my_tui_app
cd my_tui_app
cargo add ratatui crossterm anyhow
```

### Step 2: Setup Terminal (Boilerplate)
In `main.rs`, we need to enter "Raw Mode" and setup the panic hook.
```rust
use std::io;
use crossterm::{endpoint::EnterAlternateScreen, terminal::{enable_raw_mode, EnterAlternateScreen}, ExecutableCommand};
use ratatui::{backend::CrosstermBackend, Terminal};

fn main() -> anyhow::Result<()> {
    // 1. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Run App (The Loop)
    loop {
        terminal.draw(|frame| {
             use ratatui::{widgets::{Block, Borders}, layout::Rect};
             let block = Block::default()
                 .title("My App")
                 .borders(Borders::ALL);
             frame.render_widget(block, frame.size());
        })?;

        // 3. Handle Events
        if crossterm::event::poll(std::time::Duration::from_millis(16))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                if key.code == crossterm::event::KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    // 4. Restore Terminal (Crucial!)
    std::io::stdout().execute(crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
```

---

---

## 🧩 6. Tutorial: Creating Custom Widgets

Ratatui's power comes from reusable widgets. Here's how to create one.

### The Widget Trait
To draw something, you implement `Widget` for your struct.

```rust
use ratatui::{widgets::Widget, buffer::Buffer, layout::Rect, style::Style};

pub struct ProgressBar {
    progress: f64, // 0.0 to 1.0
    color: Style,
}

impl ProgressBar {
    pub fn new(progress: f64) -> Self {
        Self { progress, color: Style::default() }
    }
}

impl Widget for ProgressBar {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // 1. Clear the area
        buf.set_style(area, self.color);

        // 2. Calculate filled width
        let width = area.width as f64 * self.progress.clamp(0.0, 1.0);
        let filled_width = width.round() as u16;

        // 3. Draw filled part
        let filled_area = Rect { width: filled_width, ..area };
        for x in filled_area.left()..filled_area.right() {
             for y in filled_area.top()..filled_area.bottom() {
                 if let Some(cell) = buf.cell_mut((x, y)) {
                     cell.set_symbol("█"); 
                 }
             }
        }
    }
}
```

### Usage
In your `draw` function:
```rust
let bar = ProgressBar::new(0.75);
frame.render_widget(bar, chunk[0]);
```

---

## 🔌 7. Tutorial: Connecting to LibSQL

### Step 1: Add Dependency
```bash
cargo add libsql tokio
```

### Step 2: Connection & Schema
```rust
use libsql::Builder;

pub struct DB {
    conn: libsql::Connection,
}

impl DB {
    pub async fn new(url: &str) -> anyhow::Result<Self> {
        let db = Builder::new_local(url).build().await?;
        let conn = db.connect()?;
        Ok(Self { conn })
    }

    pub async fn init(&self) -> anyhow::Result<()> {
        self.conn.execute("
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )
        ", ()).await?;
        Ok(())
    }
}
```

### Step 3: Querying
```rust
impl DB {
    pub async fn add_user(&self, name: &str) -> anyhow::Result<()> {
        self.conn.execute(
            "INSERT INTO users (name) VALUES (?)",
            [name] // Parametrized query!
        ).await?;
        Ok(())
    }

    pub async fn get_users(&self) -> anyhow::Result<Vec<String>> {
        let mut rows = self.conn.query("SELECT name FROM users", ()).await?;
        let mut names = Vec::new();
        
        while let Some(row) = rows.next().await? {
            names.push(row.get(0)?);
        }
        Ok(names)
    }
}
```

---

## 🏆 8. Best Practices for Rust Development

### 1. Project Layout
*   `src/main.rs`: Only entry point logic (setup terminal, run loop).
*   `src/app.rs`: The "Controller". Holds the update loop and draw calls.
*   `src/state.rs`: The "Model". Pure data.
*   `src/action.rs`: The "Events". Enums for user intent.
*   `src/ui.rs`: (Optional) If drawing gets complex, move widgets here.

### 2. Error Handling
*   **Don't Unwrap**: Use `?` operator.
*   **Context**: Use `context("Failed to initialize DB")?` from `anyhow` to add info to errors.

### 3. State Management
*   **Type State**: Use enums to represent mutually exclusive states.
    ```rust
    enum Screen {
        Dashboard,
        Settings(SettingsState), // Data only exists when in Settings
    }
    ```

### 4. Git & Commits
*   **Conventional Commits**: `feat: add settings`, `fix: overflow bug`.
*   **Clean Diffs**: `cargo fmt` before every commit.
*   **Clippy**: Run `cargo clippy` often. It teaches you better Rust.
