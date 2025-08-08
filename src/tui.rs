use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap,
    },
    Frame, Terminal,
};
use std::{collections::HashMap, io};

use crate::{config::Config, parser::Keybinding, theme::parse_hex_color};

pub struct App {
    keybindings: Vec<Keybinding>,
    filtered_keybindings: Vec<(usize, Keybinding)>, // (original_index, keybinding)
    categories: HashMap<String, Vec<usize>>, // category -> indices into keybindings
    search_query: String,
    list_state: ListState,
    show_help: bool,
    config: Config,
    matcher: SkimMatcherV2,
    columns: usize, // Number of columns to display
    column_lists: Vec<ListState>, // List states for each column
}

impl App {
    pub fn new(keybindings: Vec<Keybinding>, config: &Config) -> Self {
        let mut app = Self {
            keybindings: keybindings.clone(),
            filtered_keybindings: keybindings.iter().enumerate().map(|(i, kb)| (i, kb.clone())).collect(),
            categories: HashMap::new(),
            search_query: String::new(),
            list_state: ListState::default(),
            show_help: false,
            config: config.clone(),
            matcher: SkimMatcherV2::default(),
            columns: 1,
            column_lists: vec![ListState::default()],
        };
        
        app.build_categories();
        app.list_state.select(Some(0));
        app.column_lists[0].select(Some(0));
        app
    }
    
    fn build_categories(&mut self) {
        self.categories.clear();
        
        for (index, keybinding) in self.keybindings.iter().enumerate() {
            self.categories
                .entry(keybinding.category.clone())
                .or_insert_with(Vec::new)
                .push(index);
        }
    }
    
    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        // Main loop
        let result = self.run_app(&mut terminal);
        
        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        
        result
    }
    
    fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;
            
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc if !self.show_help => break,
                        KeyCode::Char('?') | KeyCode::F(1) => {
                            self.show_help = !self.show_help;
                        }
                        KeyCode::Esc if self.show_help => {
                            self.show_help = false;
                        }
                        KeyCode::Down | KeyCode::Char('j') if !self.show_help => {
                            self.next();
                        }
                        KeyCode::Up | KeyCode::Char('k') if !self.show_help => {
                            self.previous();
                        }
                        KeyCode::Char(c) if !self.show_help => {
                            self.search_query.push(c);
                            self.filter_keybindings();
                        }
                        KeyCode::Backspace if !self.show_help => {
                            self.search_query.pop();
                            self.filter_keybindings();
                        }
                        KeyCode::Enter if !self.show_help => {
                            // Copy selected keybinding to clipboard or show details
                            if let Some(selected) = self.list_state.selected() {
                                if let Some((_, _keybinding)) = self.filtered_keybindings.get(selected) {
                                    // For now, just continue - could implement clipboard copying here
                                    continue;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }
    
    fn calculate_columns(&mut self, terminal_width: u16) {
        // Calculate optimal number of columns based on terminal width
        // Minimum width per column: 50 characters (allows for reasonable keybinding display)
        let min_column_width = 50;
        let new_columns = ((terminal_width as usize).saturating_sub(4) / min_column_width).max(1);
        
        if new_columns != self.columns {
            self.columns = new_columns;
            self.column_lists = vec![ListState::default(); self.columns];
            
            // Set initial selection in first column
            if !self.filtered_keybindings.is_empty() {
                self.column_lists[0].select(Some(0));
            }
        }
    }
    
    fn filter_keybindings(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_keybindings = self.keybindings
                .iter()
                .enumerate()
                .map(|(i, kb)| (i, kb.clone()))
                .collect();
        } else {
            let mut matches: Vec<(usize, Keybinding, i64)> = self.keybindings
                .iter()
                .enumerate()
                .filter_map(|(i, kb)| {
                    let search_text = format!("{} {} {}", kb.key, kb.action, kb.description);
                    if let Some(score) = self.matcher.fuzzy_match(&search_text, &self.search_query) {
                        Some((i, kb.clone(), score))
                    } else {
                        None
                    }
                })
                .collect();
            
            // Sort by score (higher is better)
            matches.sort_by(|a, b| b.2.cmp(&a.2));
            
            // Take up to max_results
            self.filtered_keybindings = matches
                .into_iter()
                .take(self.config.ui.max_results)
                .map(|(i, kb, _)| (i, kb))
                .collect();
        }
        
        // Reset selections for all columns
        for column_list in &mut self.column_lists {
            column_list.select(None);
        }
        
        if !self.filtered_keybindings.is_empty() {
            self.list_state.select(Some(0));
            if !self.column_lists.is_empty() {
                self.column_lists[0].select(Some(0));
            }
        } else {
            self.list_state.select(None);
        }
    }
    
    fn get_items_per_column(&self) -> usize {
        if self.columns == 0 {
            return 0;
        }
        (self.filtered_keybindings.len() + self.columns - 1) / self.columns
    }
    
    fn get_current_column(&self) -> usize {
        if let Some(selected) = self.list_state.selected() {
            let items_per_column = self.get_items_per_column();
            if items_per_column > 0 {
                selected / items_per_column
            } else {
                0
            }
        } else {
            0
        }
    }
    
    fn get_current_row_in_column(&self) -> usize {
        if let Some(selected) = self.list_state.selected() {
            let items_per_column = self.get_items_per_column();
            if items_per_column > 0 {
                selected % items_per_column
            } else {
                0
            }
        } else {
            0
        }
    }
    
    fn update_column_selection(&mut self) {
        let current_column = self.get_current_column();
        let current_row = self.get_current_row_in_column();
        
        // Clear all column selections
        for column_list in &mut self.column_lists {
            column_list.select(None);
        }
        
        // Set selection in current column
        if current_column < self.column_lists.len() {
            self.column_lists[current_column].select(Some(current_row));
        }
    }
    
    fn next(&mut self) {
        if self.filtered_keybindings.is_empty() {
            return;
        }
        
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.filtered_keybindings.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.update_column_selection();
    }
    
    fn previous(&mut self) {
        if self.filtered_keybindings.is_empty() {
            return;
        }
        
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_keybindings.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
        self.update_column_selection();
    }
    
    fn ui(&mut self, f: &mut Frame) {
        if self.show_help {
            self.render_help(f);
            return;
        }
        
        // Calculate columns based on terminal width
        self.calculate_columns(f.area().width);
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Search bar
                Constraint::Min(0),    // List
                Constraint::Length(2), // Status bar
            ])
            .split(f.area());
        
        // Search bar
        let theme = self.config.ui.theme.colors.clone();
        let search_block = Block::default()
            .borders(Borders::ALL)
            .title("Search Keybindings")
            .border_style(Style::default().fg(parse_hex_color(&theme.border_color)));
        
        let search_text = if self.search_query.is_empty() {
            "Type to search... (? for help, q to quit)".to_string()
        } else {
            self.search_query.clone()
        };
        
        let search_paragraph = Paragraph::new(search_text)
            .block(search_block)
            .style(if self.search_query.is_empty() {
                Style::default().fg(parse_hex_color(&theme.description_color))
            } else {
                Style::default().fg(parse_hex_color(&theme.search_fg))
            });
        
        f.render_widget(search_paragraph, chunks[0]);
        
        // Render keybindings in columns
        self.render_keybindings_columns(f, chunks[1]);
        
        // Status bar
        let status_text = if let Some(selected) = self.list_state.selected() {
            if let Some((_, kb)) = self.filtered_keybindings.get(selected) {
                if self.config.ui.show_raw_command {
                    format!("Raw: {}", kb.raw_command)
                } else {
                    format!("Category: {} | Action: {}", kb.category, kb.action)
                }
            } else {
                "No selection".to_string()
            }
        } else {
            "No keybindings found".to_string()
        };
        
        let status_paragraph = Paragraph::new(status_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(parse_hex_color(&theme.border_color))))
            .style(Style::default().fg(parse_hex_color(&theme.description_color)));
        
        f.render_widget(status_paragraph, chunks[2]);
    }
    
    fn render_keybindings_columns(&mut self, f: &mut Frame, area: Rect) {
        if self.columns == 1 {
            // Single column - use the original rendering
            self.render_single_column(f, area);
        } else {
            // Multiple columns
            self.render_multiple_columns(f, area);
        }
    }
    
    fn render_single_column(&mut self, f: &mut Frame, area: Rect) {
        let theme = self.config.ui.theme.colors.clone();
        let items: Vec<ListItem> = self.filtered_keybindings
            .iter()
            .map(|(_, kb)| self.create_list_item(kb, &theme))
            .collect();
        
        let list_title = format!(
            "Keybindings ({}/{})",
            self.filtered_keybindings.len(),
            self.keybindings.len()
        );
        
        let list = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title(list_title)
                .border_style(Style::default().fg(parse_hex_color(&theme.border_color))))
            .highlight_style(Style::default()
                .bg(parse_hex_color(&theme.selected_bg))
                .fg(parse_hex_color(&theme.selected_fg)))
            .highlight_symbol("> ");
        
        f.render_stateful_widget(list, area, &mut self.list_state);
    }
    
    fn render_multiple_columns(&mut self, f: &mut Frame, area: Rect) {
        // Create column constraints
        let column_constraints: Vec<Constraint> = (0..self.columns)
            .map(|_| Constraint::Percentage(100 / self.columns as u16))
            .collect();
        
        let column_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(column_constraints)
            .split(area);
        
        let items_per_column = self.get_items_per_column();
        let theme = self.config.ui.theme.colors.clone();
        let filtered_len = self.filtered_keybindings.len();
        let total_len = self.keybindings.len();
        let columns = self.columns;
        
        for (col_idx, &chunk) in column_chunks.iter().enumerate() {
            if col_idx >= columns {
                break;
            }
            
            let start_idx = col_idx * items_per_column;
            let end_idx = ((col_idx + 1) * items_per_column).min(filtered_len);
            
            if start_idx < filtered_len {
                let column_items: Vec<ListItem> = self.filtered_keybindings[start_idx..end_idx]
                    .iter()
                    .map(|(_, kb)| self.create_list_item(kb, &theme))
                    .collect();
                
                let list_title = if col_idx == 0 {
                    format!(
                        "Keybindings ({}/{}) - {} columns",
                        filtered_len,
                        total_len,
                        columns
                    )
                } else {
                    "".to_string()
                };
                
                let list = List::new(column_items)
                    .block(Block::default()
                        .borders(if col_idx == 0 { Borders::ALL } else { Borders::TOP | Borders::BOTTOM | Borders::RIGHT })
                        .title(list_title)
                        .border_style(Style::default().fg(parse_hex_color(&theme.border_color))))
                    .highlight_style(Style::default()
                        .bg(parse_hex_color(&theme.selected_bg))
                        .fg(parse_hex_color(&theme.selected_fg)))
                    .highlight_symbol("> ");
                
                if col_idx < self.column_lists.len() {
                    f.render_stateful_widget(list, chunk, &mut self.column_lists[col_idx]);
                } else {
                    f.render_widget(list, chunk);
                }
            }
        }
    }
    
    fn create_list_item<'a>(&self, kb: &'a Keybinding, theme: &crate::config::ThemeColors) -> ListItem<'a> {
        let key_style = Style::default()
            .fg(parse_hex_color(&theme.key_color))
            .add_modifier(Modifier::BOLD);
        let category_style = Style::default()
            .fg(parse_hex_color(&theme.category_color));
        let description_style = Style::default()
            .fg(parse_hex_color(&theme.action_color));
        
        let content = if self.config.ui.show_descriptions && !kb.description.is_empty() {
            vec![
                Line::from(vec![
                    Span::styled(&kb.key, key_style),
                    Span::raw(" → "),
                    Span::styled(&kb.description, description_style),
                ]),
                Line::from(vec![
                    Span::raw("  "),
                    Span::styled(&kb.category, category_style),
                ]),
            ]
        } else {
            vec![Line::from(vec![
                Span::styled(&kb.key, key_style),
                Span::raw(" → "),
                Span::styled(&kb.action, description_style),
            ])]
        };
        
        ListItem::new(content)
    }
    
    fn render_help(&self, f: &mut Frame) {
        let area = f.area();
        
        // Create a centered popup
        let popup_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(area)[1];
        
        let popup_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(popup_area)[1];
        
        f.render_widget(Clear, popup_area);
        
        let help_text = vec![
            "Hypr-showkey Help",
            "",
            "Navigation:",
            "  ↑/k       - Move up",
            "  ↓/j       - Move down",
            "  Enter     - Select keybinding",
            "",
            "Search:",
            "  Type      - Search keybindings",
            "  Backspace - Delete search character",
            "",
            "Display:",
            "  Auto      - Columns adapt to terminal width",
            "            - Min 50 chars per column",
            "            - Unbound keys are filtered out",
            "",
            "General:",
            "  ?/F1      - Toggle this help",
            "  Esc       - Close help/Clear search",
            "  q         - Quit application",
            "",
            "Search supports fuzzy matching across:",
            "- Key combinations",
            "- Action descriptions",
            "- Categories",
            "",
            "Press Esc to close this help.",
        ];
        
        let help_paragraph = Paragraph::new(help_text.join("\n"))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Help")
                    .title_alignment(Alignment::Center)
            )
            .wrap(Wrap { trim: true })
            .style(Style::default().fg(Color::White));
        
        f.render_widget(help_paragraph, popup_area);
    }
}