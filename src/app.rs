use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Step {
    Languages,
    Databases,
    AiTools,
    ExtraTools,
    Gitignore,
    Summary,
}

impl Step {
    pub fn next(self) -> Option<Step> {
        match self {
            Step::Languages => Some(Step::Databases),
            Step::Databases => Some(Step::AiTools),
            Step::AiTools => Some(Step::ExtraTools),
            Step::ExtraTools => Some(Step::Gitignore),
            Step::Gitignore => Some(Step::Summary),
            Step::Summary => None,
        }
    }

    pub fn prev(self) -> Option<Step> {
        match self {
            Step::Languages => None,
            Step::Databases => Some(Step::Languages),
            Step::AiTools => Some(Step::Databases),
            Step::ExtraTools => Some(Step::AiTools),
            Step::Gitignore => Some(Step::ExtraTools),
            Step::Summary => Some(Step::Gitignore),
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Step::Languages => "Languages & Runtimes",
            Step::Databases => "Databases",
            Step::AiTools => "AI Coding Tools",
            Step::ExtraTools => "Extra CLI Tools",
            Step::Gitignore => "Gitignore Patterns",
            Step::Summary => "Summary",
        }
    }

    pub fn prompt(self) -> &'static str {
        match self {
            Step::Languages => "Which languages or runtimes does this project use?",
            Step::Databases => "Which databases will this project use?",
            Step::AiTools => "Which AI coding tools would you like installed?",
            Step::ExtraTools => "Which extra CLI tools should be included?",
            Step::Gitignore => "Which patterns should be added to .gitignore?",
            Step::Summary => "Review your selections — Enter to create, Esc to cancel.",
        }
    }

    pub fn number(self) -> usize {
        match self {
            Step::Languages => 1,
            Step::Databases => 2,
            Step::AiTools => 3,
            Step::ExtraTools => 4,
            Step::Gitignore => 5,
            Step::Summary => 6,
        }
    }

    pub const TOTAL: usize = 6;
}

#[derive(Debug, Clone)]
pub struct SelectItem {
    pub label: &'static str,
    pub hint: &'static str,
    pub selected: bool,
}

impl SelectItem {
    pub fn new(label: &'static str, hint: &'static str) -> Self {
        Self { label, hint, selected: false }
    }

    pub fn on(mut self) -> Self {
        self.selected = true;
        self
    }
}

#[derive(Debug, Clone)]
pub struct SelectList {
    pub items: Vec<SelectItem>,
    pub cursor: usize,
}

impl SelectList {
    pub fn new(items: Vec<SelectItem>) -> Self {
        Self { items, cursor: 0 }
    }

    pub fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.cursor + 1 < self.items.len() {
            self.cursor += 1;
        }
    }

    pub fn toggle(&mut self) {
        if let Some(item) = self.items.get_mut(self.cursor) {
            item.selected = !item.selected;
        }
    }

    pub fn selected_labels(&self) -> Vec<&'static str> {
        self.items.iter().filter(|i| i.selected).map(|i| i.label).collect()
    }
}

#[derive(Debug, Clone)]
pub struct Answers {
    pub project_name: String,
    pub languages: Vec<&'static str>,
    pub databases: Vec<&'static str>,
    pub ai_tools: Vec<&'static str>,
    pub extra_tools: Vec<&'static str>,
    pub gitignore: Vec<&'static str>,
}

pub struct App {
    pub step: Step,
    pub project_name: String,
    pub languages: SelectList,
    pub databases: SelectList,
    pub ai_tools: SelectList,
    pub extra_tools: SelectList,
    pub gitignore: SelectList,
}

impl App {
    pub fn new(project_name: String) -> Self {
        Self {
            step: Step::Languages,
            project_name,
            languages: SelectList::new(vec![
                SelectItem::new("Rust", "systems / CLI"),
                SelectItem::new("Python", "via uv"),
                SelectItem::new("Node.js / Bun", "JS / TS"),
                SelectItem::new("Go", "golang"),
                SelectItem::new("Ruby", "rbenv"),
                SelectItem::new("Java / JVM", "default-jdk + maven"),
            ]),
            databases: SelectList::new(vec![
                SelectItem::new("PostgreSQL", "libpq + psql client"),
                SelectItem::new("MySQL / MariaDB", "mysql client"),
                SelectItem::new("SQLite", "embedded SQL"),
                SelectItem::new("Redis", "in-memory store"),
                SelectItem::new("MongoDB", "document database"),
            ]),
            ai_tools: SelectList::new(vec![
                SelectItem::new("Claude Code", "Anthropic claude CLI").on(),
                SelectItem::new("GitHub Copilot", "gh copilot extension"),
                SelectItem::new("OpenCode", "opencode.ai CLI"),
            ]),
            extra_tools: SelectList::new(vec![
                SelectItem::new("ripgrep", "fast grep").on(),
                SelectItem::new("fd-find", "fast find").on(),
                SelectItem::new("jq", "JSON processor").on(),
                SelectItem::new("trash-cli", "safe delete").on(),
                SelectItem::new("bat", "syntax-highlighted cat"),
                SelectItem::new("delta", "pretty git diffs"),
                SelectItem::new("htop", "process viewer"),
                SelectItem::new("httpie", "HTTP client"),
                SelectItem::new("just", "command runner"),
                SelectItem::new("watchexec", "file watcher"),
            ]),
            gitignore: SelectList::new(vec![
                SelectItem::new(".env", "environment secrets").on(),
                SelectItem::new(".env.local", "local env overrides").on(),
                SelectItem::new("*.log", "log files").on(),
                SelectItem::new(".DS_Store", "macOS metadata").on(),
                SelectItem::new("*.tmp / *.swp", "temp and editor files").on(),
                SelectItem::new("target/", "Rust build output"),
                SelectItem::new("node_modules/", "Node packages"),
                SelectItem::new(".venv/ / venv/", "Python virtualenvs"),
                SelectItem::new("__pycache__/ / *.pyc", "Python bytecode"),
                SelectItem::new("dist/ / build/", "build artifacts"),
                SelectItem::new(".idea/ / .vscode/", "IDE settings"),
            ]),
        }
    }

    pub fn active_list_mut(&mut self) -> Option<&mut SelectList> {
        match self.step {
            Step::Languages => Some(&mut self.languages),
            Step::Databases => Some(&mut self.databases),
            Step::AiTools => Some(&mut self.ai_tools),
            Step::ExtraTools => Some(&mut self.extra_tools),
            Step::Gitignore => Some(&mut self.gitignore),
            Step::Summary => None,
        }
    }

    pub fn answers(&self) -> Answers {
        Answers {
            project_name: self.project_name.clone(),
            languages: self.languages.selected_labels(),
            databases: self.databases.selected_labels(),
            ai_tools: self.ai_tools.selected_labels(),
            extra_tools: self.extra_tools.selected_labels(),
            gitignore: self.gitignore.selected_labels(),
        }
    }
}

pub fn run_wizard(project_name: &str) -> Result<Option<Answers>, Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new(project_name.to_string());

    let result = event_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    match result {
        Ok(true) => Ok(Some(app.answers())),
        Ok(false) => Ok(None),
        Err(e) => Err(e),
    }
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<bool, Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|f| crate::ui::render(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                return Ok(false);
            }

            if app.step == Step::Summary {
                match key.code {
                    KeyCode::Enter | KeyCode::Char('y') => return Ok(true),
                    KeyCode::Esc | KeyCode::Char('n') | KeyCode::Char('q') => return Ok(false),
                    KeyCode::Left | KeyCode::Char('b') => {
                        app.step = app.step.prev().unwrap_or(Step::Summary);
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') => {
                        if let Some(list) = app.active_list_mut() {
                            list.move_up();
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if let Some(list) = app.active_list_mut() {
                            list.move_down();
                        }
                    }
                    KeyCode::Char(' ') => {
                        if let Some(list) = app.active_list_mut() {
                            list.toggle();
                        }
                    }
                    KeyCode::Enter | KeyCode::Right => {
                        if let Some(next) = app.step.next() {
                            app.step = next;
                        } else {
                            return Ok(true);
                        }
                    }
                    KeyCode::Esc | KeyCode::Left | KeyCode::Char('b') => {
                        match app.step.prev() {
                            Some(prev) => app.step = prev,
                            None => return Ok(false),
                        }
                    }
                    KeyCode::Char('q') => return Ok(false),
                    _ => {}
                }
            }
        }
    }
}
