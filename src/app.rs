use std::str::FromStr;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::prelude::*;
use ratatui::widgets::*;
use rust_decimal::Decimal;

use crate::action::Action;
use crate::database::DB;
use crate::error::Result;
use crate::models::CategoryName;
use crate::state::{ActiveInput, ActiveTab, InputMode, State};
use crate::tui::{self, Tui};

/// Format a Decimal as IDR currency (e.g., 1000000 -> "1.000.000")
fn format_idr(amount: Decimal) -> String {
    let num = amount.trunc().to_string();
    let is_negative = num.starts_with('-');
    let digits: String = num.chars().filter(|c| c.is_ascii_digit()).collect();

    if digits.is_empty() {
        return "0".to_string();
    }

    let mut result = String::new();
    for (i, c) in digits.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push('.');
        }
        result.push(c);
    }

    let formatted: String = result.chars().rev().collect();
    if is_negative {
        format!("-{}", formatted)
    } else {
        formatted
    }
}

/// Main application struct
pub struct App {
    db: DB,
    state: State,
    should_quit: bool,
}

impl App {
    pub async fn new() -> Result<Self> {
        let db = DB::new().await?;
        let mut state = State::new();
        state.categories = db.get_categories().await?;
        state.balances = db.get_category_balances().await?;

        // Load initial report data
        state.transactions = db.get_transactions(state.report_date_range).await?;
        state.summary_stats = Some(db.get_summary_stats().await?);

        Ok(Self {
            db,
            state,
            should_quit: false,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = tui::restore();
            original_hook(panic_info);
        }));

        let mut terminal = tui::init()?;
        let result = self.run_loop(&mut terminal).await;
        tui::restore()?;
        result
    }

    async fn run_loop(&mut self, terminal: &mut Tui) -> Result<()> {
        while !self.should_quit {
            self.draw(terminal)?;
            if let Some(action) = self.handle_events()? {
                self.update(action).await?;
            }
        }
        Ok(())
    }

    fn draw(&mut self, terminal: &mut Tui) -> Result<()> {
        terminal.draw(|frame| {
            let area = frame.area();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                    Constraint::Length(3),
                ])
                .split(area);

            self.draw_header(frame, layout[0]);
            self.draw_content(frame, layout[1]);
            self.draw_footer(frame, layout[2]);

            if self.state.show_help {
                self.draw_help_overlay(frame, area);
            }
        })?;
        Ok(())
    }

    fn draw_header(&self, frame: &mut Frame, area: Rect) {
        let tabs: Vec<Line> = ActiveTab::all()
            .iter()
            .map(|t| {
                if *t == self.state.active_tab {
                    Line::from(format!(" {} ", t.title())).style(
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Line::from(format!(" {} ", t.title()))
                }
            })
            .collect();

        let tabs_widget = Tabs::new(tabs)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Ebisu - Kakeibo Tracker "),
            )
            .select(self.state.active_tab as usize)
            .highlight_style(Style::default().fg(Color::Yellow));

        frame.render_widget(tabs_widget, area);
    }

    fn draw_content(&mut self, frame: &mut Frame, area: Rect) {
        match self.state.active_tab {
            ActiveTab::Dashboard => self.draw_dashboard(frame, area),
            ActiveTab::AddFunds => self.draw_add_funds(frame, area),
            ActiveTab::AddExpense => self.draw_add_expense(frame, area),
            ActiveTab::Reports => self.draw_reports(frame, area),
            ActiveTab::Settings => self.draw_settings(frame, area),
        }
    }

    fn draw_dashboard(&self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(area);

        let balance_items: Vec<ListItem> = self
            .state
            .balances
            .iter()
            .map(|b| {
                let spent_pct = if b.allocated > Decimal::ZERO {
                    (b.spent / b.allocated * Decimal::from(100))
                        .round_dp(1)
                        .to_string()
                } else {
                    "0".to_string()
                };
                let remaining = b.available - b.spent;
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{:<12}", b.category_name),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::raw(format!(
                        " IDR {:>12} / IDR {:>12} ({:>5}%)",
                        format_idr(remaining),
                        format_idr(b.allocated),
                        spent_pct
                    )),
                ]))
            })
            .collect();

        let balances_list = List::new(balance_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Category Balances "),
            )
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        frame.render_widget(balances_list, layout[0]);

        let total_available: Decimal = self.state.balances.iter().map(|b| b.available).sum();
        let total_spent: Decimal = self.state.balances.iter().map(|b| b.spent).sum();
        let total_remaining = total_available - total_spent;

        let savings = self
            .state
            .balances
            .iter()
            .find(|b| b.category_name == CategoryName::Savings)
            .map(|b| b.available - b.spent)
            .unwrap_or_default();

        let summary_text = vec![
            Line::from(vec![
                Span::styled("Total Available: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("IDR {}", format_idr(total_available)),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(vec![
                Span::styled("Total Spent:     ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("IDR {}", format_idr(total_spent)),
                    Style::default().fg(Color::Red),
                ),
            ]),
            Line::from(vec![
                Span::styled("Remaining:       ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("IDR {}", format_idr(total_remaining)),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Savings:         ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("IDR {}", format_idr(savings)),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        let summary = Paragraph::new(summary_text)
            .block(Block::default().borders(Borders::ALL).title(" Summary "));

        frame.render_widget(summary, layout[1]);
    }

    fn draw_add_funds(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).title(" Add Funds ");
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(inner);

        let input_style = if self.state.active_input == ActiveInput::Amount {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };

        let amount_input = Paragraph::new(self.state.amount_input.as_str())
            .style(input_style)
            .block(Block::default().borders(Borders::ALL).title(" Amount "));
        frame.render_widget(amount_input, layout[0]);

        let instructions = Paragraph::new("Press i to type, Enter to submit, Esc to cancel")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(instructions, layout[2]);
    }

    fn draw_add_expense(&mut self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Add Expense ");
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(6),
                Constraint::Min(0),
            ])
            .split(inner);

        let amount_style = if self.state.active_input == ActiveInput::Amount {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let amount_input = Paragraph::new(self.state.amount_input.as_str())
            .style(amount_style)
            .block(Block::default().borders(Borders::ALL).title(" Amount "));
        frame.render_widget(amount_input, layout[0]);

        let desc_style = if self.state.active_input == ActiveInput::Description {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default()
        };
        let desc_input = Paragraph::new(self.state.description_input.as_str())
            .style(desc_style)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Description "),
            );
        frame.render_widget(desc_input, layout[1]);

        let selectable_cats: Vec<&crate::models::Category> = self
            .state
            .categories
            .iter()
            .filter(|c| c.name != CategoryName::Savings)
            .collect();

        let cat_items: Vec<ListItem> = selectable_cats
            .iter()
            .enumerate()
            .map(|(i, c)| {
                let style = if i == self.state.selected_category {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(format!(" {} ", c.name)).style(style)
            })
            .collect();

        let cat_list = List::new(cat_items)
            .block(Block::default().borders(Borders::ALL).title(" Category "))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        frame.render_widget(cat_list, layout[2]);

        let instructions = Paragraph::new("Tab: switch fields | Up/Down: category | Enter: submit")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(instructions, layout[3]);
    }

    fn draw_reports(&self, frame: &mut Frame, area: Rect) {
        // Layout: Stats on top, Transactions list below
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7), // Summary Stats
                Constraint::Min(0),    // Transactions
            ])
            .split(area);

        // --- Summary Stats ---
        let date_range_title = format!(" < {} > ", self.state.report_date_range.title());
        let stats_block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" Period Stats: {} ", date_range_title))
            .title_alignment(Alignment::Center);

        if let Some(stats) = &self.state.summary_stats {
            let inner_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .margin(1)
                .split(stats_block.inner(layout[0]));

            frame.render_widget(stats_block, layout[0]);

            let left_stats = vec![
                Line::from(vec![
                    Span::styled("Total Funds Added: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("IDR {}", format_idr(stats.total_funds_added)),
                        Style::default().fg(Color::Green),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Total Expenses:    ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("IDR {}", format_idr(stats.total_spent)),
                        Style::default().fg(Color::Red),
                    ),
                ]),
            ];
            frame.render_widget(Paragraph::new(left_stats), inner_layout[0]);

            // Right side could show savings rate or overflow count if tracked?
            // For now just show net flow maybe?
            let net = stats.total_funds_added - stats.total_spent;
            let net_color = if net >= Decimal::ZERO {
                Color::Cyan
            } else {
                Color::Red
            };

            let right_stats = vec![Line::from(vec![
                Span::styled("Net Flow:          ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("IDR {}", format_idr(net)),
                    Style::default().fg(net_color).add_modifier(Modifier::BOLD),
                ),
            ])];
            frame.render_widget(Paragraph::new(right_stats), inner_layout[1]);
        } else {
            frame.render_widget(stats_block, layout[0]);
            frame.render_widget(Paragraph::new("Loading..."), layout[0]);
        }

        // --- Transactions List ---
        let list_block = Block::default()
            .borders(Borders::ALL)
            .title(" Transaction History ");

        let items: Vec<ListItem> = self
            .state
            .transactions
            .iter()
            .map(|t| {
                let date_str = t.created_at.format("%Y-%m-%d %H:%M").to_string();
                let cat_name = t
                    .category_name
                    .map(|c| c.to_string())
                    .unwrap_or("Unknown".to_string());

                let desc = t.description.clone().unwrap_or_default();
                let desc_display = if desc.len() > 20 {
                    format!("{}...", &desc[0..17])
                } else {
                    desc
                };

                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("{:<18}", date_str),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::raw(" | "),
                    Span::styled(
                        format!("{:<12}", cat_name),
                        Style::default().fg(Color::Cyan),
                    ),
                    Span::raw(" | "),
                    Span::styled(
                        format!("IDR {:>12}", format_idr(t.amount)),
                        Style::default().fg(Color::White),
                    ),
                    Span::raw(" | "),
                    Span::styled(desc_display, Style::default().fg(Color::Gray)),
                ]))
            })
            .collect();

        let list = List::new(items).block(list_block);
        frame.render_widget(list, layout[1]);
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        let mode_str = match self.state.input_mode {
            InputMode::Normal => "NORMAL",
            InputMode::Insert => "INSERT",
        };

        let status = self
            .state
            .status_message
            .clone()
            .unwrap_or_else(|| "Ready".to_string());

        let footer_text = Line::from(vec![
            Span::styled(
                format!(" {} ", mode_str),
                Style::default()
                    .bg(Color::Blue)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(status, Style::default().fg(Color::Gray)),
            Span::raw(" | "),
            Span::styled("? for Help", Style::default().fg(Color::DarkGray)),
        ]);

        let footer = Paragraph::new(footer_text).block(Block::default().borders(Borders::ALL));
        frame.render_widget(footer, area);
    }

    fn draw_settings(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Settings - Category Allocation ");
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(inner);

        // Category List
        let items: Vec<ListItem> = self
            .state
            .categories
            .iter()
            .enumerate()
            .map(|(i, cat)| {
                let is_selected = i == self.state.selected_category;
                // Check if this category is being edited
                let is_editing = self.state.input_mode == InputMode::Insert
                    && self.state.active_input == ActiveInput::CategoryLimit
                    && is_selected;

                let limit_display = if is_editing {
                    format!("{}%", self.state.amount_input)
                } else {
                    format!("{}%", cat.limit_percentage)
                };

                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let prefix = if is_selected { "> " } else { "  " };
                let edit_indicator = if is_editing { " [EDITING]" } else { "" };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("{}{:<15}", prefix, cat.name), style),
                    Span::raw(" | "),
                    Span::styled(format!("{:>6}", limit_display), style),
                    Span::styled(edit_indicator, Style::default().fg(Color::Green)),
                ]))
            })
            .collect();

        let list =
            List::new(items).highlight_style(Style::default().add_modifier(Modifier::REVERSED));
        // We handle selection manual rendering above for more control

        frame.render_widget(list, layout[0]);

        // Total Percentage Check
        let total_percent: Decimal = self
            .state
            .categories
            .iter()
            .map(|c| c.limit_percentage)
            .sum();
        let total_color = if total_percent == Decimal::new(100, 0) {
            Color::Green
        } else {
            Color::Red
        };

        let info_text = vec![
            Line::from(vec![
                Span::raw("Total Allocation: "),
                Span::styled(
                    format!("{}%", total_percent),
                    Style::default().fg(total_color),
                ),
            ]),
            Line::from(vec![
                Span::styled("Enter/i", Style::default().fg(Color::Yellow)),
                Span::raw(": Edit  "),
                Span::styled("Esc", Style::default().fg(Color::Yellow)),
                Span::raw(": Cancel  "),
                Span::styled("Enter", Style::default().fg(Color::Yellow)),
                Span::raw(": Save"),
            ]),
        ];
        frame.render_widget(Paragraph::new(info_text), layout[1]);
    }

    fn draw_help_overlay(&self, frame: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from("Navigation:"),
            Line::from("  Tab/Shift+Tab  Switch tabs"),
            Line::from("  Up/Down        Navigate lists"),
            Line::from(""),
            Line::from("Input:"),
            Line::from("  i              Enter insert mode"),
            Line::from("  Esc            Exit insert mode"),
            Line::from("  Enter          Submit form"),
            Line::from(""),
            Line::from("General:"),
            Line::from("  ?              Toggle help"),
            Line::from("  q              Quit application"),
        ];

        let help_block = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Help ")
                    .style(Style::default().bg(Color::DarkGray)),
            )
            .alignment(Alignment::Left);

        let popup_area = centered_rect(50, 60, area);
        frame.render_widget(Clear, popup_area);
        frame.render_widget(help_block, popup_area);
    }

    fn handle_events(&mut self) -> Result<Option<Action>> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    return Ok(None);
                }

                if key.code == KeyCode::Char('q') && self.state.input_mode == InputMode::Normal {
                    return Ok(Some(Action::Quit));
                }

                if key.code == KeyCode::Char('?') && self.state.input_mode == InputMode::Normal {
                    return Ok(Some(Action::ToggleHelp));
                }

                match self.state.input_mode {
                    InputMode::Normal => return self.handle_normal_mode(key),
                    InputMode::Insert => return self.handle_insert_mode(key),
                }
            }
        }
        Ok(None)
    }

    fn handle_normal_mode(&mut self, key: event::KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Tab => {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    Ok(Some(Action::PrevTab))
                } else {
                    Ok(Some(Action::NextTab))
                }
            }
            KeyCode::Char('i') => Ok(Some(Action::EnterInsert)),
            KeyCode::Up | KeyCode::Char('k') => Ok(Some(Action::Up)),
            KeyCode::Down | KeyCode::Char('j') => Ok(Some(Action::Down)),
            KeyCode::Enter => match self.state.active_tab {
                ActiveTab::AddFunds => Ok(Some(Action::SubmitFunds)),
                ActiveTab::AddExpense => Ok(Some(Action::SubmitTransaction)),
                ActiveTab::Settings => Ok(Some(Action::StartEditingCategory)),
                _ => Ok(None),
            },
            KeyCode::Left | KeyCode::Char('h') => {
                if self.state.active_tab == ActiveTab::Reports {
                    Ok(Some(Action::ChangeDateRange(
                        self.state.report_date_range.next(),
                    ))) // Cycling for now
                } else {
                    Ok(None)
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if self.state.active_tab == ActiveTab::Reports {
                    Ok(Some(Action::ChangeDateRange(
                        self.state.report_date_range.next(),
                    )))
                } else {
                    Ok(None)
                }
            }
            _ => Ok(None),
        }
    }

    fn handle_insert_mode(&mut self, key: event::KeyEvent) -> Result<Option<Action>> {
        match key.code {
            KeyCode::Esc => {
                if self.state.input_mode == InputMode::Insert
                    && self.state.active_input == ActiveInput::CategoryLimit
                {
                    Ok(Some(Action::CancelInput))
                } else {
                    Ok(Some(Action::EnterNormal))
                }
            }
            KeyCode::Enter => match self.state.active_tab {
                ActiveTab::AddFunds => Ok(Some(Action::SubmitFunds)),
                ActiveTab::AddExpense => Ok(Some(Action::SubmitTransaction)),
                ActiveTab::Settings => Ok(Some(Action::SaveCategoryLimit)),
                _ => Ok(Some(Action::EnterNormal)),
            },
            KeyCode::Tab => {
                match self.state.active_input {
                    ActiveInput::Amount => {
                        self.state.active_input = ActiveInput::Description;
                    }
                    ActiveInput::Description => {
                        self.state.active_input = ActiveInput::Category;
                    }
                    ActiveInput::Category => {
                        self.state.active_input = ActiveInput::Amount;
                    }
                    ActiveInput::None => {
                        self.state.active_input = ActiveInput::Amount;
                    }
                    ActiveInput::CategoryLimit => {} // No tab cycling in limit edit
                }
                Ok(None)
            }
            KeyCode::Char(c) => Ok(Some(Action::InputChar(c))),
            KeyCode::Backspace => Ok(Some(Action::InputBackspace)),
            KeyCode::Delete => Ok(Some(Action::InputDelete)),
            KeyCode::Up => Ok(Some(Action::Up)),
            KeyCode::Down => Ok(Some(Action::Down)),
            _ => Ok(None),
        }
    }

    async fn update(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Quit => {
                self.should_quit = true;
            }
            Action::NextTab => {
                self.state.active_tab = self.state.active_tab.next();
                self.state.clear_inputs();
                if self.state.active_tab == ActiveTab::Reports {
                    // Auto-refresh reports on tab switch
                    self.state.transactions = self
                        .db
                        .get_transactions(self.state.report_date_range)
                        .await?;
                    self.state.summary_stats = Some(self.db.get_summary_stats().await?);
                }
            }
            Action::PrevTab => {
                self.state.active_tab = self.state.active_tab.prev();
                self.state.clear_inputs();
                if self.state.active_tab == ActiveTab::Reports {
                    self.state.transactions = self
                        .db
                        .get_transactions(self.state.report_date_range)
                        .await?;
                    self.state.summary_stats = Some(self.db.get_summary_stats().await?);
                }
            }
            Action::EnterInsert => {
                self.state.input_mode = InputMode::Insert;
                if self.state.active_input == ActiveInput::None {
                    self.state.active_input = ActiveInput::Amount;
                }
            }
            Action::EnterNormal => {
                self.state.input_mode = InputMode::Normal;
            }
            Action::ToggleHelp => {
                self.state.show_help = !self.state.show_help;
            }
            Action::StartEditingCategory => {
                if let Some(cat) = self.state.categories.get(self.state.selected_category) {
                    self.state.input_mode = InputMode::Insert;
                    self.state.active_input = ActiveInput::CategoryLimit;
                    self.state.amount_input = cat.limit_percentage.to_string();
                }
            }
            Action::SaveCategoryLimit => {
                if let Ok(limit) = Decimal::from_str(&self.state.amount_input) {
                    if let Some(cat) = self.state.categories.get(self.state.selected_category) {
                        match self.db.update_category_limit(cat.id, limit).await {
                            Ok(_) => {
                                self.state.set_status(format!(
                                    "Updated {} limit to {}%",
                                    cat.name, limit
                                ));
                                self.state.clear_inputs();
                                self.state.categories = self.db.get_categories().await?;
                            }
                            Err(e) => self.state.set_status(format!("Error: {}", e)),
                        }
                    }
                } else {
                    self.state.set_status("Invalid number format");
                }
            }
            Action::CancelInput => {
                self.state.clear_inputs();
            }
            Action::Up => {
                if self.state.active_tab == ActiveTab::Settings {
                    if self.state.selected_category > 0 {
                        self.state.selected_category -= 1;
                    }
                } else if self.state.active_input == ActiveInput::Category
                    && self.state.selected_category > 0
                {
                    self.state.selected_category -= 1;
                }
            }
            Action::Down => {
                let max_idx = self
                    .state
                    .categories
                    .iter()
                    .filter(|c| {
                        c.name != CategoryName::Savings
                            || self.state.active_tab == ActiveTab::Settings
                    })
                    .count()
                    .saturating_sub(1);

                if (self.state.active_tab == ActiveTab::Settings
                    || self.state.active_input == ActiveInput::Category)
                    && self.state.selected_category < max_idx
                {
                    self.state.selected_category += 1;
                }
            }
            Action::InputChar(c) => match self.state.active_input {
                ActiveInput::Amount | ActiveInput::CategoryLimit => {
                    if c.is_ascii_digit() || c == '.' {
                        self.state.amount_input.push(c);
                    }
                }
                ActiveInput::Description => {
                    self.state.description_input.push(c);
                }
                _ => {}
            },
            Action::InputBackspace => match self.state.active_input {
                ActiveInput::Amount | ActiveInput::CategoryLimit => {
                    self.state.amount_input.pop();
                }
                ActiveInput::Description => {
                    self.state.description_input.pop();
                }
                _ => {}
            },
            Action::InputDelete => match self.state.active_input {
                ActiveInput::Amount | ActiveInput::CategoryLimit => {
                    self.state.amount_input.pop();
                }
                ActiveInput::Description => {
                    self.state.description_input.pop();
                }
                _ => {}
            },
            Action::SubmitFunds => {
                if let Ok(amount) = Decimal::from_str(&self.state.amount_input) {
                    if amount > Decimal::ZERO {
                        match self.db.add_funds(amount).await {
                            Ok(_) => {
                                self.state.set_status(format!(
                                    "Added IDR {} in funds",
                                    format_idr(amount)
                                ));
                                self.state.clear_inputs();
                                self.state.balances = self.db.get_category_balances().await?;
                            }
                            Err(e) => {
                                self.state.set_status(format!("Error: {}", e));
                            }
                        }
                    } else {
                        self.state.set_status("Amount must be positive");
                    }
                } else {
                    self.state.set_status("Invalid amount format");
                }
            }
            Action::SubmitTransaction => {
                if let Ok(amount) = Decimal::from_str(&self.state.amount_input) {
                    if amount > Decimal::ZERO {
                        let selectable_cats: Vec<&crate::models::Category> = self
                            .state
                            .categories
                            .iter()
                            .filter(|c| c.name != CategoryName::Savings)
                            .collect();

                        if let Some(cat) = selectable_cats.get(self.state.selected_category) {
                            let desc = if self.state.description_input.is_empty() {
                                None
                            } else {
                                Some(self.state.description_input.clone())
                            };

                            match self.db.create_transaction(cat.name, amount, desc).await {
                                Ok(_) => {
                                    self.state.set_status(format!(
                                        "Added IDR {} expense to {}",
                                        format_idr(amount),
                                        cat.name
                                    ));
                                    self.state.clear_inputs();
                                    self.state.balances = self.db.get_category_balances().await?;
                                }
                                Err(e) => {
                                    self.state.set_status(format!("Error: {}", e));
                                }
                            }
                        }
                    } else {
                        self.state.set_status("Amount must be positive");
                    }
                } else {
                    self.state.set_status("Invalid amount format");
                }
            }
            Action::RefreshBalances => {
                self.state.balances = self.db.get_category_balances().await?;
            }
            Action::RefreshCategories => {
                self.state.categories = self.db.get_categories().await?;
            }
            Action::ChangeDateRange(range) => {
                self.state.report_date_range = range;
                self.state.transactions = self.db.get_transactions(range).await?;
                // Summary stats are global for now
                self.state.summary_stats = Some(self.db.get_summary_stats().await?);
            }
            _ => {}
        }
        Ok(())
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
