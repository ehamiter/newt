//! Application state and initialization.

use super::types::{Answers, SelectItem, SelectList, Step};

/// Main application state for the wizard.
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
    /// Create a new wizard with default selections.
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
                SelectItem::new("Swift", "swiftly toolchain (linux)"),
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
                SelectItem::new("pi", "@mariozechner/pi-coding-agent"),
            ]),
            extra_tools: SelectList::new(vec![
                SelectItem::new("ripgrep", "fast grep").on(),
                SelectItem::new("fd-find", "fast find").on(),
                SelectItem::new("jq", "JSON processor").on(),
                SelectItem::new("trash-cli", "safe delete").on(),
                SelectItem::new("bat", "syntax-highlighted cat"),
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

    /// Get mutable reference to the currently active selection list.
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

    /// Collect all answers into a final configuration.
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