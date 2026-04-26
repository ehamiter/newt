//! Terminal UI rendering.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Step};

/// Minimum terminal size required.
pub const MIN_WIDTH: u16 = 60;
pub const MIN_HEIGHT: u16 = 15;

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    // Check if terminal is large enough
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        render_too_small(f, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

    render_header(f, app, chunks[0]);

    if app.step == Step::Summary {
        render_summary(f, app, chunks[1]);
    } else {
        render_checklist(f, app, chunks[1]);
    }

    render_footer(f, app.step, chunks[2]);
}

/// Render message when terminal is too small.
fn render_too_small(f: &mut Frame, area: Rect) {
    let message = "Terminal too small. Please resize.";
    let para = Paragraph::new(message)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(" Error ")
            .style(Style::default().fg(Color::Red)))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(para, area);
}

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    // Progress bar showing steps
    let progress = (0..Step::TOTAL)
        .map(|i| {
            let step_num = i + 1;
            if step_num < app.step.number() {
                "●"  // Completed
            } else if step_num == app.step.number() {
                "○"  // Current
            } else {
                "·"  // Pending
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let title = format!(
        " newt — {} — {}/{} ",
        app.project_name,
        app.step.number(),
        Step::TOTAL
    );

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Cyan));

    // Create layout with progress indicator
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(40),
            Constraint::Length(15),
        ])
        .split(area);

    let para = Paragraph::new(app.step.prompt())
        .block(block)
        .style(Style::default().fg(Color::White));
    f.render_widget(para, chunks[0]);

    // Progress indicator
    let progress_para = Paragraph::new(progress)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(progress_para, chunks[1]);
}

fn render_checklist(f: &mut Frame, app: &App, area: Rect) {
    let list = match app.step {
        Step::Languages => &app.languages,
        Step::Databases => &app.databases,
        Step::AiTools => &app.ai_tools,
        Step::ExtraTools => &app.extra_tools,
        Step::Gitignore => &app.gitignore,
        Step::Summary => unreachable!(),
    };

    let items: Vec<ListItem> = list
        .items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_cursor = i == list.cursor;

            let check_style = if item.selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let label_style = if is_cursor {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if item.selected {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Gray)
            };

            let prefix = if is_cursor { "▶ " } else { "  " };
            let checkbox = if item.selected { "[x] " } else { "[ ] " };

            ListItem::new(Line::from(vec![
                Span::raw(prefix),
                Span::styled(checkbox, check_style),
                Span::styled(item.label, label_style),
                Span::styled(
                    format!("  — {}", item.hint),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();

    let title = format!(" {} ", app.step.title());
    let list_widget = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(list_widget, area);
}

fn render_summary(f: &mut Frame, app: &App, area: Rect) {
    let answers = app.answers();
    let mut lines: Vec<Line<'static>> = Vec::new();

    lines.push(Line::from(vec![
        Span::styled(
            "  Project:  ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            answers.project_name.clone(),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::raw(""));

    add_section(&mut lines, "Languages", &answers.languages);
    add_section(&mut lines, "Databases", &answers.databases);
    add_section(&mut lines, "AI Tools", &answers.ai_tools);
    add_section(&mut lines, "Extra Tools", &answers.extra_tools);
    add_section(&mut lines, "Gitignore", &answers.gitignore);

    let para = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title(" Summary "))
        .wrap(Wrap { trim: false });

    f.render_widget(para, area);
}

fn add_section(lines: &mut Vec<Line<'static>>, label: &'static str, items: &[&'static str]) {
    lines.push(Line::from(vec![Span::styled(
        format!("  {}:  ", label),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )]));

    if items.is_empty() {
        lines.push(Line::from(Span::styled(
            "    (none)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for &item in items {
            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled("• ", Style::default().fg(Color::Green)),
                Span::styled(item, Style::default().fg(Color::White)),
            ]));
        }
    }

    lines.push(Line::raw(""));
}

fn render_footer(f: &mut Frame, step: Step, area: Rect) {
    let (hint, style) = if step == Step::Summary {
        (
            " [Enter/y] Create   [n/Esc] Cancel   [←/b] Back ",
            Style::default().fg(Color::Green),
        )
    } else {
        (
            " [↑↓/jk] Move   [Space] Toggle   [a] Select All   [A] Deselect All   [Enter/→] Next   [←/b/Esc] Back   [q/Ctrl+C] Quit ",
            Style::default().fg(Color::DarkGray),
        )
    };

    let para = Paragraph::new(hint)
        .block(Block::default().borders(Borders::ALL))
        .style(style)
        .alignment(Alignment::Center);

    f.render_widget(para, area);
}