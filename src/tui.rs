use std::io;
use std::time::Duration;

use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use tokio::sync::mpsc;

use polymarket_client_sdk::gamma;
use polymarket_client_sdk::gamma::types::request::MarketsRequest;
use polymarket_client_sdk::gamma::types::response::Market;

const MENU_ITEMS: &[&str] = &["Markets"];

#[derive(PartialEq)]
enum View {
    Menu,
    Markets,
}

type MarketsResult = std::result::Result<Vec<Market>, String>;

struct App {
    view: View,
    menu_state: ListState,
    markets: Vec<Market>,
    markets_state: ListState,
    loading: bool,
    error: Option<String>,
    should_quit: bool,
    markets_rx: Option<mpsc::Receiver<MarketsResult>>,
}

impl App {
    fn new() -> Self {
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));
        Self {
            view: View::Menu,
            menu_state,
            markets: Vec::new(),
            markets_state: ListState::default(),
            loading: false,
            error: None,
            should_quit: false,
            markets_rx: None,
        }
    }

    async fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|f| self.render(f))?;
            self.poll_events()?;
            self.poll_async_results();
        }
        Ok(())
    }

    fn render(&mut self, f: &mut Frame) {
        let area = f.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(area);

        let title_text = match self.view {
            View::Menu => "polyterm".to_string(),
            View::Markets => "polyterm › markets".to_string(),
        };
        let title = Paragraph::new(title_text)
            .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            .block(Block::default().borders(Borders::BOTTOM));
        f.render_widget(title, chunks[0]);

        match self.view {
            View::Menu => self.render_menu(f, chunks[1]),
            View::Markets => self.render_markets(f, chunks[1]),
        }

        let hints = match self.view {
            View::Menu => " j/k: move   enter: open   q: quit ",
            View::Markets => " j/k: scroll   esc: back   q: quit ",
        };
        let footer = Paragraph::new(hints).style(Style::default().fg(Color::DarkGray));
        f.render_widget(footer, chunks[2]);
    }

    fn render_menu(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = MENU_ITEMS.iter().map(|m| ListItem::new(*m)).collect();
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(" menu "))
            .highlight_style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
        f.render_stateful_widget(list, area, &mut self.menu_state);
    }

    fn render_markets(&mut self, f: &mut Frame, area: Rect) {
        if self.loading {
            let p = Paragraph::new("Loading markets...").alignment(Alignment::Center);
            f.render_widget(p, area);
            return;
        }
        if let Some(err) = &self.error {
            let p = Paragraph::new(format!("Error: {err}"))
                .style(Style::default().fg(Color::Red))
                .wrap(Wrap { trim: true })
                .block(Block::default().borders(Borders::ALL).title(" markets "));
            f.render_widget(p, area);
            return;
        }

        let question_width: usize = area.width.saturating_sub(24).max(20) as usize;
        let items: Vec<ListItem> = self
            .markets
            .iter()
            .map(|m| {
                let q = m.question.as_deref().unwrap_or("—");
                let liq = m
                    .liquidity
                    .map(|l| l.to_string().parse::<f64>().unwrap_or(0.0))
                    .map(|l| format!("${:>10.0}", l))
                    .unwrap_or_else(|| "         —".to_string());
                let line = format!("{:<width$}  {}", truncate(q, question_width), liq, width = question_width);
                ListItem::new(line)
            })
            .collect();

        let title = format!(" markets ({}) ", self.markets.len());
        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("> ");
        f.render_stateful_widget(list, area, &mut self.markets_state);
    }

    fn poll_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.on_key(key.code, key.modifiers);
            }
        }
        Ok(())
    }

    fn on_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
            self.should_quit = true;
            return;
        }
        match self.view {
            View::Menu => match code {
                KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                KeyCode::Char('j') | KeyCode::Down => {
                    step(&mut self.menu_state, MENU_ITEMS.len(), 1);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    step(&mut self.menu_state, MENU_ITEMS.len(), -1);
                }
                KeyCode::Enter => self.open_selected_menu(),
                _ => {}
            },
            View::Markets => match code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('h') => {
                    self.view = View::Menu;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    step(&mut self.markets_state, self.markets.len(), 1);
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    step(&mut self.markets_state, self.markets.len(), -1);
                }
                _ => {}
            },
        }
    }

    fn open_selected_menu(&mut self) {
        let Some(idx) = self.menu_state.selected() else {
            return;
        };
        match MENU_ITEMS.get(idx) {
            Some(&"Markets") => {
                self.view = View::Markets;
                self.fetch_markets();
            }
            _ => {}
        }
    }

    fn fetch_markets(&mut self) {
        self.loading = true;
        self.error = None;
        self.markets.clear();
        self.markets_state.select(None);

        let (tx, rx) = mpsc::channel(1);
        self.markets_rx = Some(rx);
        tokio::spawn(async move {
            let client = gamma::Client::default();
            let req = MarketsRequest::builder()
                .maybe_closed(Some(false))
                .limit(500)
                .maybe_order(Some("volume_num".to_string()))
                .ascending(false)
                .build();
            let result = client.markets(&req).await.map_err(|e| e.to_string());
            let _ = tx.send(result).await;
        });
    }

    fn poll_async_results(&mut self) {
        let Some(rx) = self.markets_rx.as_mut() else {
            return;
        };
        match rx.try_recv() {
            Ok(Ok(markets)) => {
                self.markets = markets;
                if !self.markets.is_empty() {
                    self.markets_state.select(Some(0));
                }
                self.loading = false;
                self.markets_rx = None;
            }
            Ok(Err(e)) => {
                self.error = Some(e);
                self.loading = false;
                self.markets_rx = None;
            }
            Err(mpsc::error::TryRecvError::Empty) => {}
            Err(mpsc::error::TryRecvError::Disconnected) => {
                self.loading = false;
                self.markets_rx = None;
            }
        }
    }
}

fn step(state: &mut ListState, len: usize, dir: i32) {
    if len == 0 {
        return;
    }
    let current = state.selected().unwrap_or(0) as i32;
    let next = (current + dir).rem_euclid(len as i32);
    state.select(Some(next as usize));
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

pub async fn run() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let result = app.run(&mut terminal).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}
