use crate::ui::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3),  // Title + Status
                Constraint::Min(0),     // Middle (Logs + Windows)
                Constraint::Length(10), // Context Pane (fixed height)
                Constraint::Length(3),  // Help
            ]
            .as_ref(),
        )
        .split(frame.size());

    // 1. Header
    let status_color = if app.status.contains("Found") || app.status.contains("Clicked") {
        Color::Green
    } else {
        Color::Yellow
    };

    let header_text = vec![Line::from(vec![
        Span::styled(
            "Ag-Accept ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("| "),
        Span::raw(format!("Target: {} | ", app.config.target_window_title)),
        Span::styled(
            format!("State: {}", app.status),
            Style::default().fg(status_color),
        ),
    ])];

    let header =
        Paragraph::new(header_text).block(Block::default().borders(Borders::ALL).title("Info"));

    frame.render_widget(header, chunks[0]);

    // 2. Middle Split
    let middle_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    // 2a. Logs (Left)
    let log_items: Vec<ListItem> = app
        .logs
        .iter()
        .rev()
        .map(|m| {
            let style = if m.contains("ERROR") {
                Style::default().fg(Color::Red)
            } else if m.contains("Clicked") {
                Style::default().fg(Color::Green)
            } else if m.contains("Found") {
                Style::default().fg(Color::Magenta)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(Line::from(Span::styled(m, style)))
        })
        .collect();

    let logs_list = List::new(log_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Logs (Newest First)"),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(logs_list, middle_chunks[0]);

    // 2b. Visible Windows (Right)
    let target_lower = app.config.target_window_title.to_lowercase();
    let processing = app.processing_window.clone().unwrap_or_default();

    let window_items: Vec<ListItem> = app
        .visible_windows
        .iter()
        .map(|w| {
            // Priority 1: Processing Cursor (Green Underline)
            if !processing.is_empty() && w == &processing {
                ListItem::new(Line::from(Span::styled(
                    format!("> {}", w),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::UNDERLINED | Modifier::BOLD),
                )))
            }
            // Priority 2: Matches Target Config (Bold)
            else if w.to_lowercase().contains(&target_lower) {
                ListItem::new(Line::from(Span::styled(
                    w,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                )))
            } else {
                ListItem::new(Line::from(w.as_str()))
            }
        })
        .collect();

    let windows_list = List::new(window_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Visible Windows"),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_widget(windows_list, middle_chunks[1]);

    // 3. Context Pane (Bottom Split) - NEW
    let context_block = Block::default().borders(Borders::ALL).title("Context Info");
    if let Some((btn, neighbors)) = &app.context_data {
        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::raw("Found Button: "),
            Span::styled(
                btn,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(""));
        lines.push(Line::from("Neighbors (Prev 2 -> Next 2):"));

        for n in neighbors {
            let style = if n.contains("*MATCH*") {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::UNDERLINED)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            lines.push(Line::from(Span::styled(n, style)));
        }

        // Only show last 6 lines if too many? No, just list them.
        let p = Paragraph::new(lines).block(context_block);
        frame.render_widget(p, chunks[2]);
    } else {
        let p =
            Paragraph::new("No context data yet. Waiting for button match...").block(context_block);
        frame.render_widget(p, chunks[2]);
    }

    // 4. Footer
    let help = Paragraph::new("Press 'q' or 'Esc' to quit.")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(help, chunks[3]);
}
