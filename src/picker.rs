use crate::fuzzy;
use crate::registry::{Skill, display_path};
use crate::search::script_search_text;
use crate::theme::{self, Colors};
use anyhow::{Context, Result, bail};
use chromata::Theme as ColorTheme;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::cursor::{Hide, Show};
use ratatui::crossterm::event::{
    self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap};
use std::fs::File;
use std::io::{self, BufRead, BufReader, IsTerminal};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;
use syntect::easy::HighlightLines;
use syntect::highlighting::{ScopeSelectors, StyleModifier, Theme, ThemeItem};
use syntect::parsing::SyntaxSet;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

struct TerminalGuard;
impl TerminalGuard {
    fn enter() -> Result<Self> {
        enable_raw_mode().context("failed to enable terminal raw mode")?;
        if let Err(error) = execute!(
            io::stderr(),
            EnterAlternateScreen,
            EnableBracketedPaste,
            EnableMouseCapture,
            Hide
        ) {
            drop(Self);
            return Err(error.into());
        }
        Ok(Self)
    }
}
impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(
            io::stderr(),
            Show,
            DisableMouseCapture,
            DisableBracketedPaste,
            LeaveAlternateScreen
        );
        let _ = disable_raw_mode();
    }
}

struct CachedCard {
    text: Text<'static>,
    height: u16,
}

struct PreviewRequest {
    generation: u64,
    path: PathBuf,
    width: u16,
}

struct PreviewResult {
    generation: u64,
    lines: Vec<Line<'static>>,
}

fn preview_worker(
    requests: Receiver<PreviewRequest>,
    results: Sender<PreviewResult>,
    theme: &'static ColorTheme,
    latest_generation: Arc<AtomicU64>,
) {
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let syntax_theme = syntax_theme(theme);
    let muted = theme::colors(theme).muted;
    while let Ok(mut request) = requests.recv() {
        while let Ok(newer) = requests.try_recv() {
            request = newer;
        }
        if let Some(lines) = preview_lines(
            &request.path,
            &syntax_set,
            &syntax_theme,
            muted,
            request.width,
            &latest_generation,
            request.generation,
        ) && results
            .send(PreviewResult {
                generation: request.generation,
                lines,
            })
            .is_err()
        {
            break;
        }
    }
}

struct App {
    items: Vec<Skill>,
    query: String,
    matches: Vec<usize>,
    selected: usize,
    first_visible: usize,
    card_scroll: u16,
    card_max_scroll: u16,
    preview_scroll: usize,
    preview_max_scroll: usize,
    preview_width: u16,
    preview_area: Option<Rect>,
    preview: Vec<Line<'static>>,
    theme: &'static ColorTheme,
    colors: Colors,
    cards: Vec<CachedCard>,
    card_width: Option<u16>,
    visible_cards: Vec<(Rect, usize)>,
    preview_generation: u64,
    latest_generation: Arc<AtomicU64>,
    preview_requests: Sender<PreviewRequest>,
    preview_results: Receiver<PreviewResult>,
}
impl App {
    fn new(items: Vec<Skill>, theme: &'static ColorTheme) -> Self {
        let matches = (0..items.len()).collect();
        let (preview_requests, requests) = mpsc::channel();
        let (results, preview_results) = mpsc::channel();
        let latest_generation = Arc::new(AtomicU64::new(0));
        let worker_generation = Arc::clone(&latest_generation);
        thread::spawn(move || preview_worker(requests, results, theme, worker_generation));
        let mut app = Self {
            items,
            query: String::new(),
            matches,
            selected: 0,
            first_visible: 0,
            card_scroll: 0,
            card_max_scroll: 0,
            preview_scroll: 0,
            preview_max_scroll: 0,
            preview_width: 0,
            preview_area: None,
            preview: Vec::new(),
            theme,
            colors: theme::colors(theme),
            cards: Vec::new(),
            card_width: None,
            visible_cards: Vec::new(),
            preview_generation: 0,
            latest_generation,
            preview_requests,
            preview_results,
        };
        app.load_preview();
        app
    }
    fn selected_item(&self) -> Option<&Skill> {
        self.matches
            .get(self.selected)
            .map(|&index| &self.items[index])
    }
    fn filter(&mut self) {
        let query = self.query.trim();
        let mut scored: Vec<(usize, i64)> = self
            .items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                if query.is_empty() {
                    Some((index, 0))
                } else {
                    fuzzy::score(&script_search_text(item), query)
                        .filter(|score| *score >= query.chars().count() as i64 * 8)
                        .map(|score| (index, score))
                }
            })
            .collect();
        if !query.is_empty() {
            scored.sort_by(|(li, ls), (ri, rs)| {
                rs.cmp(ls)
                    .then_with(|| self.items[*li].name.cmp(&self.items[*ri].name))
            });
        }
        self.matches = scored.into_iter().map(|(index, _)| index).collect();
        self.selected = 0;
        self.first_visible = 0;
        self.card_scroll = 0;
        self.card_max_scroll = 0;
        self.cards.clear();
        self.card_width = None;
        self.load_preview();
    }
    fn move_selection(&mut self, amount: isize) {
        if self.matches.is_empty() {
            return;
        }
        let len = self.matches.len() as isize;
        let next = (self.selected as isize + amount).rem_euclid(len) as usize;
        self.select(next);
    }
    fn select(&mut self, position: usize) -> bool {
        if position >= self.matches.len() || position == self.selected {
            return false;
        }
        self.selected = position;
        self.card_scroll = 0;
        self.load_preview();
        true
    }
    fn load_preview(&mut self) {
        self.preview_generation = self.preview_generation.wrapping_add(1);
        self.latest_generation
            .store(self.preview_generation, Ordering::Relaxed);
        self.preview_scroll = 0;
        self.preview_max_scroll = 0;
        self.preview = if self.preview_width > 0
            && let Some(item) = self.selected_item()
        {
            let _ = self.preview_requests.send(PreviewRequest {
                generation: self.preview_generation,
                path: item.path.clone(),
                width: self.preview_width,
            });
            vec![Line::from("Loading preview…")]
        } else {
            Vec::new()
        };
    }
    fn set_preview_width(&mut self, width: u16) {
        if self.preview_width != width {
            self.preview_width = width;
            self.load_preview();
        }
    }
    fn scroll_preview(&mut self, amount: isize) -> bool {
        let next = self
            .preview_scroll
            .saturating_add_signed(amount)
            .min(self.preview_max_scroll);
        if next == self.preview_scroll {
            return false;
        }
        self.preview_scroll = next;
        true
    }
    fn receive_preview(&mut self) -> bool {
        let mut changed = false;
        while let Ok(result) = self.preview_results.try_recv() {
            if result.generation == self.preview_generation {
                self.preview = result.lines;
                changed = true;
            }
        }
        changed
    }
    fn ensure_cards(&mut self, width: u16) {
        if self.card_width == Some(width) {
            return;
        }
        let colors = self.colors;
        self.cards = self
            .matches
            .iter()
            .map(|&index| {
                let text = card_text(&self.items[index], &self.query, colors);
                let height = card_height_from_text(&text, width);
                CachedCard { text, height }
            })
            .collect();
        self.card_width = Some(width);
    }
    fn ensure_visible(&mut self, height: u16) {
        if self.matches.is_empty() || height == 0 {
            return;
        }
        if self.selected < self.first_visible {
            self.first_visible = self.selected;
        }
        let mut used = 0u16;
        for position in (self.first_visible..=self.selected).rev() {
            used = used.saturating_add(self.cards[position].height.saturating_add(1));
            if used > height {
                self.first_visible = if position == self.selected {
                    position
                } else {
                    position + 1
                };
                break;
            }
        }
    }
}

pub(crate) fn run_picker(items: Vec<Skill>, theme: &'static ColorTheme) -> Result<Option<Skill>> {
    if items.is_empty() {
        return Ok(None);
    }
    if !io::stdin().is_terminal() || !io::stderr().is_terminal() {
        bail!("interactive picker requires a terminal");
    }
    let guard = TerminalGuard::enter()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stderr()))?;
    let result = run_loop(&mut terminal, App::new(items, theme));
    drop(terminal);
    drop(guard);
    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stderr>>,
    mut app: App,
) -> Result<Option<Skill>> {
    let mut redraw = true;
    loop {
        redraw |= app.receive_preview();
        if redraw {
            terminal.draw(|frame| draw(frame, &mut app))?;
            redraw = false;
        }
        if !event::poll(Duration::from_millis(30))? {
            continue;
        }
        let changed = match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match handle_key(&mut app, key) {
                Action::Continue => true,
                Action::Choose => return Ok(app.selected_item().cloned()),
                Action::Cancel => return Ok(None),
            },
            Event::Paste(text) => {
                app.query.push_str(&text.replace(['\r', '\n'], " "));
                app.filter();
                true
            }
            Event::Mouse(mouse) => handle_mouse(&mut app, mouse),
            Event::Resize(_, _) => true,
            _ => false,
        };
        redraw |= changed;
    }
}

fn handle_mouse(app: &mut App, mouse: MouseEvent) -> bool {
    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            let selected = app.visible_cards.iter().find_map(|(area, position)| {
                contains(*area, mouse.column, mouse.row).then_some(*position)
            });
            selected.is_some_and(|position| app.select(position))
        }
        MouseEventKind::ScrollDown
            if app
                .preview_area
                .is_some_and(|area| contains(area, mouse.column, mouse.row)) =>
        {
            app.scroll_preview(3)
        }
        MouseEventKind::ScrollUp
            if app
                .preview_area
                .is_some_and(|area| contains(area, mouse.column, mouse.row)) =>
        {
            app.scroll_preview(-3)
        }
        _ => false,
    }
}

fn contains(area: Rect, column: u16, row: u16) -> bool {
    column >= area.x && column < area.right() && row >= area.y && row < area.bottom()
}

enum Action {
    Continue,
    Choose,
    Cancel,
}
fn handle_key(app: &mut App, key: KeyEvent) -> Action {
    match (key.code, key.modifiers) {
        (KeyCode::Esc, _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => Action::Cancel,
        (KeyCode::Enter, _) => Action::Choose,
        (KeyCode::Up, KeyModifiers::ALT) => {
            app.card_scroll = app.card_scroll.saturating_sub(1);
            Action::Continue
        }
        (KeyCode::Down, KeyModifiers::ALT) => {
            app.card_scroll = app.card_scroll.saturating_add(1).min(app.card_max_scroll);
            Action::Continue
        }
        (KeyCode::Up, KeyModifiers::NONE) => {
            app.move_selection(-1);
            Action::Continue
        }
        (KeyCode::Down, KeyModifiers::NONE) => {
            app.move_selection(1);
            Action::Continue
        }
        (KeyCode::Char('u'), KeyModifiers::CONTROL) => {
            app.query.clear();
            app.filter();
            Action::Continue
        }
        (KeyCode::Char('d'), KeyModifiers::CONTROL) => {
            app.scroll_preview(10);
            Action::Continue
        }
        (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
            app.scroll_preview(-10);
            Action::Continue
        }
        (KeyCode::Backspace, _) => {
            app.query.pop();
            app.filter();
            Action::Continue
        }
        (KeyCode::Char(ch), modifiers)
            if modifiers.is_empty() || modifiers == KeyModifiers::SHIFT =>
        {
            app.query.push(ch);
            app.filter();
            Action::Continue
        }
        _ => Action::Continue,
    }
}

fn draw(frame: &mut ratatui::Frame, app: &mut App) {
    let area = frame.area();
    let colors = app.colors;
    app.visible_cards.clear();
    app.preview_area = None;
    frame.render_widget(Clear, area);
    frame.render_widget(
        Block::default().style(Style::default().bg(colors.background)),
        area,
    );
    if area.width < 35 || area.height < 9 {
        app.set_preview_width(0);
        frame.render_widget(
            Paragraph::new("Terminal too small for the picker")
                .style(Style::default().fg(colors.text)),
            area,
        );
        return;
    }
    let (list_area, preview_area) = if area.width >= 72 {
        let parts = Layout::horizontal([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(area);
        (parts[0], Some(parts[1]))
    } else {
        (area, None)
    };
    if preview_area.is_none() {
        app.set_preview_width(0);
    }
    let parts = Layout::vertical([
        Constraint::Min(3),
        Constraint::Length(3),
        Constraint::Length(3),
    ])
    .split(list_area);
    draw_list(frame, app, parts[0], colors);
    draw_help(frame, app, parts[1], colors);
    draw_query(frame, app, parts[2], colors);
    if let Some(area) = preview_area {
        draw_preview(frame, app, area, colors);
    }
}

fn pane(title: String, border: Color, background: Color) -> Block<'static> {
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border))
        .style(Style::default().bg(background))
}

fn draw_list(frame: &mut ratatui::Frame, app: &mut App, area: Rect, colors: Colors) {
    let block = pane(
        format!(" SCRIPTS  {} / {} ", app.matches.len(), app.items.len()),
        colors.border,
        colors.panel,
    )
    .title_style(Style::default().fg(colors.accent));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if app.matches.is_empty() {
        frame.render_widget(
            Paragraph::new(" No matching scripts").style(Style::default().fg(colors.muted)),
            inner,
        );
        return;
    }
    app.ensure_cards(inner.width);
    app.ensure_visible(inner.height);
    let mut y = inner.y;
    for position in app.first_visible..app.matches.len() {
        if y >= inner.bottom() {
            break;
        }
        let card = &app.cards[position];
        let full_height = card.height;
        let height = full_height.min(inner.bottom() - y);
        if height == 0 {
            break;
        }
        let card_area = Rect::new(inner.x, y, inner.width, height);
        app.visible_cards.push((card_area, position));
        let selected = position == app.selected;
        if selected {
            app.card_max_scroll = full_height.saturating_sub(height);
            app.card_scroll = app.card_scroll.min(app.card_max_scroll);
        }
        let block = pane(
            String::new(),
            if selected {
                colors.accent
            } else {
                colors.card_border
            },
            if selected {
                colors.selected
            } else {
                colors.card
            },
        );
        let body = block.inner(card_area);
        frame.render_widget(block, card_area);
        frame.render_widget(
            Paragraph::new(card.text.clone())
                .wrap(Wrap { trim: false })
                .scroll((if selected { app.card_scroll } else { 0 }, 0))
                .style(Style::default().fg(colors.text)),
            body,
        );
        y = y.saturating_add(height).saturating_add(1);
    }
}

fn card_height_from_text(text: &Text<'static>, width: u16) -> u16 {
    let width = usize::from(width.saturating_sub(2).max(1));
    text.lines
        .iter()
        .map(|line| wrapped_lines(line, width))
        .sum::<usize>()
        .saturating_add(2)
        .min(u16::MAX as usize) as u16
}

fn wrapped_lines(line: &Line<'_>, width: usize) -> usize {
    let plain: String = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect();
    let mut rows = 1;
    let mut used = 0;
    for (index, word) in plain.split(' ').enumerate() {
        let word_width = Line::from(word).width();
        let space = usize::from(index > 0 && used > 0);
        if used + space + word_width <= width {
            used += space + word_width;
        } else if word_width <= width {
            rows += 1;
            used = word_width;
        } else {
            if used > 0 {
                rows += 1;
            }
            rows += word_width.div_ceil(width) - 1;
            used = ((word_width - 1) % width) + 1;
        }
    }
    rows
}
fn card_text(item: &Skill, query: &str, colors: Colors) -> Text<'static> {
    let mut name = vec![Span::styled("› ", Style::default().fg(colors.accent))];
    name.extend(highlight(
        item.name.as_str(),
        query,
        colors,
        colors.accent,
        true,
    ));
    let tags = if item.tags.is_empty() {
        "—".to_string()
    } else {
        item.tags.join("  ·  ")
    };
    Text::from(vec![
        Line::from(name),
        field_line(
            "PATH",
            &display_path(&item.path),
            query,
            colors,
            colors.path_label,
        ),
        field_line(
            "COMMAND",
            &item.command,
            query,
            colors,
            colors.command_label,
        ),
        field_line(
            "COMMENT",
            item.comment.as_deref().unwrap_or("—"),
            query,
            colors,
            colors.comment_label,
        ),
        field_line("TAGS", &tags, query, colors, colors.tags_label),
    ])
}
fn field_line(
    label: &str,
    value: &str,
    query: &str,
    colors: Colors,
    label_color: Color,
) -> Line<'static> {
    let mut spans = vec![Span::styled(
        format!("{label:<8} "),
        Style::default().fg(label_color),
    )];
    spans.extend(highlight(value, query, colors, colors.text, false));
    Line::from(spans)
}
fn highlight(
    value: &str,
    query: &str,
    colors: Colors,
    base_color: Color,
    bold: bool,
) -> Vec<Span<'static>> {
    let chars: Vec<char> = value.chars().collect();
    let mut marked = vec![false; chars.len()];
    for term in query.split_whitespace() {
        if let Some((_, indices)) = fuzzy::match_indices(value, term) {
            for index in indices {
                marked[index] = true;
            }
        }
    }
    if chars.is_empty() {
        return Vec::new();
    }
    let mut spans = Vec::new();
    let mut start = 0;
    for index in 1..=chars.len() {
        if index == chars.len() || marked[index] != marked[start] {
            let segment: String = chars[start..index].iter().collect();
            let style = if marked[start] {
                Style::default()
                    .fg(colors.match_text)
                    .bg(colors.match_background)
                    .add_modifier(Modifier::BOLD)
            } else if bold {
                Style::default().fg(base_color).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(base_color)
            };
            spans.push(Span::styled(segment, style));
            start = index;
        }
    }
    spans
}

fn draw_help(frame: &mut ratatui::Frame, app: &App, area: Rect, colors: Colors) {
    let key = Style::default()
        .fg(colors.help_key)
        .add_modifier(Modifier::BOLD);
    let description = Style::default().fg(colors.muted);
    let mut help = if area.width >= 72 {
        vec![
            Span::styled(" ↑↓/click ", key),
            Span::styled("Alt↑↓ ", key),
            Span::styled("card  ", description),
            Span::styled("Wheel ", key),
            Span::styled("preview  ", description),
            Span::styled("Enter  Esc  ", key),
        ]
    } else {
        vec![
            Span::styled(" ↑↓/click  Enter  Esc  ", key),
            Span::styled("Theme: ", description),
        ]
    };
    help.push(Span::styled(
        app.theme.name.to_string(),
        Style::default().fg(colors.accent),
    ));
    let block = pane(String::new(), colors.border, colors.panel);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(Line::from(help)), inner);
}
fn draw_query(frame: &mut ratatui::Frame, app: &App, area: Rect, colors: Colors) {
    let block = pane(String::new(), colors.accent, colors.panel);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let content = Line::from(vec![
        Span::styled("🔎  ", Style::default().fg(colors.accent)),
        Span::styled(app.query.clone(), Style::default().fg(colors.text)),
    ]);
    let width = content.width();
    let offset = width.saturating_sub(inner.width.saturating_sub(1) as usize);
    frame.render_widget(
        Paragraph::new(content).scroll((0, offset.min(u16::MAX as usize) as u16)),
        inner,
    );
    let cursor_x = inner
        .x
        .saturating_add(width.min(inner.width.saturating_sub(1) as usize) as u16);
    frame.set_cursor_position((cursor_x, inner.y));
}
fn draw_preview(frame: &mut ratatui::Frame, app: &mut App, area: Rect, colors: Colors) {
    let title = app.selected_item().map_or_else(
        || " PREVIEW ".to_string(),
        |item| format!(" PREVIEW  {} ", item.name),
    );
    let block = pane(title, colors.card_border, colors.card)
        .title_style(Style::default().fg(colors.accent));
    let inner = block.inner(area);
    frame.render_widget(block, area);
    app.preview_area = Some(area);
    app.set_preview_width(inner.width);
    app.preview_max_scroll = app.preview.len().saturating_sub(inner.height as usize);
    app.preview_scroll = app.preview_scroll.min(app.preview_max_scroll);
    let start = app.preview_scroll;
    let end = start
        .saturating_add(usize::from(inner.height))
        .min(app.preview.len());
    frame.render_widget(
        Paragraph::new(Text::from(app.preview[start..end].to_vec()))
            .style(Style::default().fg(colors.text)),
        inner,
    );
}

fn preview_lines(
    path: &Path,
    syntax_set: &SyntaxSet,
    syntax_theme: &Theme,
    muted: Color,
    width: u16,
    latest_generation: &AtomicU64,
    generation: u64,
) -> Option<Vec<Line<'static>>> {
    if latest_generation.load(Ordering::Relaxed) != generation {
        return None;
    }
    let Ok(file) = File::open(path) else {
        return Some(vec![Line::from(format!(
            "Cannot read {}",
            display_path(path)
        ))]);
    };
    let mut reader = BufReader::new(file);
    let syntax = syntax_set
        .find_syntax_for_file(path)
        .ok()
        .flatten()
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text());
    let mut highlighter = HighlightLines::new(syntax, syntax_theme);
    let mut lines = Vec::new();
    let mut raw = Vec::new();
    let mut source_line = 0usize;
    loop {
        if latest_generation.load(Ordering::Relaxed) != generation {
            return None;
        }
        raw.clear();
        match reader.read_until(b'\n', &mut raw) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => {
                return Some(vec![Line::from(format!(
                    "Cannot read {}",
                    display_path(path)
                ))]);
            }
        }
        if latest_generation.load(Ordering::Relaxed) != generation {
            return None;
        }
        source_line += 1;
        let line = String::from_utf8_lossy(&raw);
        let spans = match highlighter.highlight_line(&line, syntax_set) {
            Ok(ranges) => ranges
                .into_iter()
                .map(|(style, content)| {
                    let color = style.foreground;
                    Span::styled(
                        content.trim_end_matches(['\r', '\n']).to_string(),
                        Style::default().fg(Color::Rgb(color.r, color.g, color.b)),
                    )
                })
                .collect(),
            Err(_) => vec![Span::raw(line.trim_end_matches(['\r', '\n']).to_string())],
        };
        lines.extend(wrap_preview_line(spans, source_line, width, muted));
    }
    if lines.is_empty() {
        lines.push(Line::from("(empty file)"));
    }
    Some(lines)
}

fn wrap_preview_line(
    spans: Vec<Span<'static>>,
    number: usize,
    width: u16,
    muted: Color,
) -> Vec<Line<'static>> {
    let prefix = format!("{number:>3}  ");
    let prefix_width = UnicodeWidthStr::width(prefix.as_str());
    let continuation = " ".repeat(prefix_width);
    let code_width = usize::from(width).saturating_sub(prefix_width).max(1);
    let prefix_style = Style::default().fg(muted);
    let mut rows = Vec::new();
    let mut current = vec![Span::styled(prefix, prefix_style)];
    let mut used = 0usize;

    for span in spans {
        let style = span.style;
        let mut segment = String::new();
        {
            let mut append = |piece: &str, piece_width: usize| {
                if used > 0 && used.saturating_add(piece_width) > code_width {
                    if !segment.is_empty() {
                        current.push(Span::styled(std::mem::take(&mut segment), style));
                    }
                    rows.push(Line::from(std::mem::take(&mut current)));
                    current.push(Span::styled(continuation.clone(), prefix_style));
                    used = 0;
                }
                segment.push_str(piece);
                used = used.saturating_add(piece_width);
            };
            for grapheme in span.content.as_ref().graphemes(true) {
                if grapheme == "\t" {
                    for _ in 0..4 {
                        append(" ", 1);
                    }
                } else {
                    append(grapheme, UnicodeWidthStr::width(grapheme));
                }
            }
        }
        if !segment.is_empty() {
            current.push(Span::styled(segment, style));
        }
    }
    rows.push(Line::from(current));
    rows
}
fn syntax_theme(theme: &ColorTheme) -> Theme {
    let mut items = Vec::new();
    for (scope, color) in [
        ("comment", theme.comment),
        ("keyword", theme.keyword),
        ("string", theme.string),
        ("entity.name.function", theme.function),
        ("variable", theme.variable),
        ("storage.type, entity.name.type", theme.r#type),
        ("constant", theme.constant),
        ("keyword.operator", theme.operator),
        ("entity.name.tag", theme.tag),
    ] {
        if let Some(color) = color {
            items.push(ThemeItem {
                scope: scope
                    .parse::<ScopeSelectors>()
                    .expect("static syntax scope"),
                style: StyleModifier {
                    foreground: Some(color.into()),
                    ..Default::default()
                },
            });
        }
    }
    Theme {
        name: Some(theme.name.to_string()),
        settings: theme.to_syntect_settings(),
        scopes: items,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::ScriptName;
    use std::fs;
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::time::{Duration, Instant};
    fn skill(name: &str, comment: &str) -> Skill {
        Skill {
            name: ScriptName::from_str(name).unwrap(),
            path: PathBuf::from("scripts/example.py"),
            registered_path: "scripts/example.py".into(),
            command: "python {{path}}".into(),
            comment: Some(comment.into()),
            tags: vec!["ase".into()],
        }
    }
    #[test]
    fn filters_and_selects_best_match() {
        let mut app = App::new(
            vec![
                skill("alpha", "other"),
                skill("convert_ase", "Convert ASE files"),
            ],
            theme::resolve(theme::DEFAULT_NAME).unwrap(),
        );
        app.query = "ase".into();
        app.filter();
        assert_eq!(app.matches.len(), 2);
        assert_eq!(app.selected_item().unwrap().name.as_str(), "convert_ase");
        app.query = "zzzzzzzzzzzz".into();
        app.filter();
        assert_eq!(app.matches.len(), 0);
    }
    #[test]
    fn card_height_grows_with_wrapped_content() {
        let colors = theme::colors(theme::resolve(theme::DEFAULT_NAME).unwrap());
        let short = skill("short", "brief");
        let long = skill("long", &"very long description ".repeat(20));
        assert!(
            card_height_from_text(&card_text(&long, "", colors), 35)
                > card_height_from_text(&card_text(&short, "", colors), 35)
        );
    }
    #[test]
    fn card_cache_survives_selection_and_rebuilds_for_query_or_width() {
        let mut app = App::new(
            vec![skill("first", "one"), skill("second", "two")],
            theme::resolve(theme::DEFAULT_NAME).unwrap(),
        );
        app.ensure_cards(40);
        let first = app.cards[0].text.lines[0].spans[1].content.as_ptr();
        app.move_selection(1);
        app.ensure_cards(40);
        assert_eq!(app.cards[0].text.lines[0].spans[1].content.as_ptr(), first);
        app.ensure_cards(35);
        assert_eq!(app.card_width, Some(35));
        app.query = "first".into();
        app.filter();
        app.ensure_cards(35);
        assert_eq!(app.cards.len(), 1);
    }
    #[test]
    fn arrow_selection_wraps_between_first_and_last_cards() {
        let mut app = App::new(
            vec![
                skill("first", "one"),
                skill("middle", "two"),
                skill("last", "three"),
            ],
            theme::resolve(theme::DEFAULT_NAME).unwrap(),
        );
        app.move_selection(-1);
        assert_eq!(app.selected_item().unwrap().name.as_str(), "last");
        app.move_selection(1);
        assert_eq!(app.selected_item().unwrap().name.as_str(), "first");
    }
    #[test]
    fn card_and_preview_use_the_same_theme() {
        let theme = theme::resolve(theme::DEFAULT_NAME).unwrap();
        let colors = theme::colors(theme);
        let preview_theme = syntax_theme(theme);
        assert_eq!(colors.background, Color::from(theme.bg));
        assert_eq!(preview_theme.settings.background, Some(theme.bg.into()));
        assert_eq!(preview_theme.settings.foreground, Some(theme.fg.into()));
    }
    #[test]
    fn escape_cancels_and_theme_stays_configured() {
        let mut app = App::new(
            vec![skill("example", "example")],
            theme::resolve(theme::DEFAULT_NAME).unwrap(),
        );
        assert!(matches!(
            handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
            Action::Cancel
        ));
        let before = app.theme;
        handle_key(
            &mut app,
            KeyEvent::new(KeyCode::Char('t'), KeyModifiers::CONTROL),
        );
        assert!(std::ptr::eq(app.theme, before));
    }

    #[test]
    fn clicking_a_card_selects_it_but_wheel_does_not() {
        let mut app = App::new(
            vec![skill("first", "one"), skill("second", "two")],
            theme::resolve(theme::DEFAULT_NAME).unwrap(),
        );
        app.visible_cards = vec![(Rect::new(1, 1, 40, 7), 0), (Rect::new(1, 9, 40, 7), 1)];
        let mouse = |kind, column, row| MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        };
        assert!(!handle_mouse(
            &mut app,
            mouse(MouseEventKind::ScrollDown, 5, 10)
        ));
        assert_eq!(app.selected, 0);
        assert!(handle_mouse(
            &mut app,
            mouse(MouseEventKind::Down(MouseButton::Left), 5, 10)
        ));
        assert_eq!(app.selected, 1);
        assert!(!handle_mouse(
            &mut app,
            mouse(MouseEventKind::ScrollUp, 5, 3)
        ));
        assert!(!handle_mouse(
            &mut app,
            mouse(MouseEventKind::Down(MouseButton::Left), 70, 10)
        ));
        assert_eq!(app.selected, 1);
    }

    #[test]
    fn wheel_scrolls_only_the_preview() {
        let mut app = App::new(
            vec![skill("first", "one")],
            theme::resolve(theme::DEFAULT_NAME).unwrap(),
        );
        app.preview_area = Some(Rect::new(70, 0, 40, 25));
        app.preview_max_scroll = 20;
        let mouse = |kind, column| MouseEvent {
            kind,
            column,
            row: 10,
            modifiers: KeyModifiers::NONE,
        };
        assert!(!handle_mouse(
            &mut app,
            mouse(MouseEventKind::ScrollDown, 5)
        ));
        assert_eq!(app.preview_scroll, 0);
        assert!(handle_mouse(
            &mut app,
            mouse(MouseEventKind::ScrollDown, 75)
        ));
        assert_eq!(app.preview_scroll, 3);
        assert!(handle_mouse(&mut app, mouse(MouseEventKind::ScrollUp, 75)));
        assert_eq!(app.preview_scroll, 0);
        assert!(!handle_mouse(&mut app, mouse(MouseEventKind::ScrollUp, 75)));
    }

    #[test]
    fn preview_wraps_code_without_losing_styles_or_line_numbers() {
        let style = Style::default().fg(Color::Red);
        let rows = wrap_preview_line(
            vec![Span::styled("abcdefghijk", style)],
            12,
            12,
            Color::Gray,
        );
        assert_eq!(
            rows.iter().map(Line::to_string).collect::<Vec<_>>(),
            [" 12  abcdefg", "     hijk",]
        );
        assert_eq!(rows[0].spans[1].style.fg, Some(Color::Red));
        assert_eq!(rows[1].spans[1].style.fg, Some(Color::Red));
        let wide = wrap_preview_line(vec![Span::raw("ab界c")], 1, 8, Color::Gray);
        assert_eq!(
            wide.iter().map(Line::to_string).collect::<Vec<_>>(),
            ["  1  ab", "     界c",]
        );
    }

    #[test]
    fn latest_preview_arrives_without_rebuilding_cards() {
        let dir = tempfile::tempdir().unwrap();
        let first_path = dir.path().join("first.py");
        let second_path = dir.path().join("second.py");
        fs::write(&first_path, "print('first source')\n").unwrap();
        fs::write(&second_path, "print('second source')\n").unwrap();
        let mut first = skill("first", "one");
        first.path = first_path;
        let mut second = skill("second", "two");
        second.path = second_path;
        let mut app = App::new(
            vec![first, second],
            theme::resolve(theme::DEFAULT_NAME).unwrap(),
        );
        app.ensure_cards(50);
        let card_text = app.cards[0].text.lines[0].spans[1].content.as_ptr();
        app.set_preview_width(25);
        app.select(1);
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline && !app.receive_preview() {
            thread::sleep(Duration::from_millis(10));
        }
        let source = app.preview.iter().map(Line::to_string).collect::<String>();
        assert!(source.contains("second source"));
        assert!(!source.contains("first source"));
        let original_rows = app.preview.len();
        app.set_preview_width(10);
        let deadline = Instant::now() + Duration::from_secs(3);
        while Instant::now() < deadline && !app.receive_preview() {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(app.preview.len() > original_rows);
        app.ensure_cards(50);
        assert_eq!(
            app.cards[0].text.lines[0].spans[1].content.as_ptr(),
            card_text
        );
    }
}
