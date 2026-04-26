//! Application state types and selection items.

use std::fmt;

/// Configuration for a selection item with label, hint, and default state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectItem {
    pub label: &'static str,
    pub hint: &'static str,
    pub selected: bool,
}

impl SelectItem {
    pub const fn new(label: &'static str, hint: &'static str) -> Self {
        Self {
            label,
            hint,
            selected: false,
        }
    }

    pub const fn on(mut self) -> Self {
        self.selected = true;
        self
    }
}

/// A list of selectable items with cursor position.
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

/// Wizard step enumeration with navigation and display metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Step {
    Languages,
    Databases,
    AiTools,
    ExtraTools,
    Gitignore,
    Summary,
}

impl Step {
    pub const TOTAL: usize = 6;

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
        self as usize + 1
    }
}

impl fmt::Display for Step {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.title())
    }
}

/// Final answers collected from the wizard.
#[derive(Debug, Clone, Default)]
pub struct Answers {
    pub project_name: String,
    pub languages: Vec<&'static str>,
    pub databases: Vec<&'static str>,
    pub ai_tools: Vec<&'static str>,
    pub extra_tools: Vec<&'static str>,
    pub gitignore: Vec<&'static str>,
}