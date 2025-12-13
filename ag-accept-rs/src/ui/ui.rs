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
                Constraint::Length(3), // Title
                Constraint::Length(3), // Stats (Timing)
                Constraint::Length(6), // Target Windows (Stats)
                Constraint::Min(5),    // Middle (Logs + All Windows) - Force visibility
                Constraint::Length(8), // Context Pane (Reduced)
                Constraint::Length(1), // Help (Reduced)
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

    // 1b. Stats Block
    let scan_color = if app.last_scan_ms < 500 {
        Color::Green
    } else if app.last_scan_ms < 1000 {
        Color::Yellow
    } else {
        Color::Red
    };

    let stats_text = vec![Line::from(vec![
        Span::raw("Last Scan: "),
        Span::styled(
            format!("{} ms", app.last_scan_ms),
            Style::default().fg(scan_color).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | Sleep Interval: "),
        Span::styled(
            format!("{:.1} s", app.sleep_interval),
            Style::default().fg(Color::Cyan),
        ),
        Span::raw(" | Mode: "),
        Span::styled(
            if app.status.contains("Sleeping") {
                "SLEEPING"
            } else {
                "ACTIVE"
            },
            Style::default().fg(if app.status.contains("Sleeping") {
                Color::Blue
            } else {
                Color::Green
            }),
        ),
    ])];

    let stats = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title("Timing Stats"));
    frame.render_widget(stats, chunks[1]);

    // 2. Target Windows (Detailed Stats)
    let processing = app.processing_window.clone().unwrap_or_default();
    let target_items: Vec<ListItem> = app
        .visible_windows
        .iter()
        .map(|w| {
            // 1. Status Column (Fixed Width 6)
            let (status_str, status_color) = if w.duration_ms > 1000 {
                ("BUSY", Color::Red)
            } else {
                ("OK", Color::Green)
            };

            // 2. Duration Column (Fixed Width 5)
            // e.g. " 120" or "1500"
            // Using {:>4} for number

            // 3. Formatting
            // "[  OK  ] [  30ms] Title"

            // 4. Processing Cursor
            let is_processing = !processing.is_empty() && w.title == processing;
            let display_title = if is_processing {
                format!("> {}", w.title)
            } else {
                w.title.clone()
            };

            // 5. Styles
            // Base style
            let mut style = Style::default();

            // Focus underline (System Focus)
            if w.is_focused {
                style = style.add_modifier(Modifier::UNDERLINED);
            }

            // Combine
            // We need spans to color JUST the status part?
            // Ratatui ListItems can take a Line which has Spans.

            let mut spans = vec![
                Span::styled("[", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:^6}", status_str),
                    Style::default()
                        .fg(status_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("] [", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("{:>4}", w.duration_ms),
                    Style::default().fg(Color::Reset),
                ), // Duration white
                Span::styled("ms] ", Style::default().fg(Color::DarkGray)),
                Span::styled(display_title, style), // Title with focus underline
            ];

            ListItem::new(Line::from(spans))
        })
        .collect();

    let targets_list = List::new(target_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Target Windows (Monitoring)"),
    );
    frame.render_widget(targets_list, chunks[2]);

    // 3. Middle Split (Logs + All Windows)
    let middle_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[3]);

    // 3a. Logs (Left)
    let log_items: Vec<ListItem> = app
        .logs
        .iter()
        .rev()
        .map(|m| {
            let style = if m.contains("ERROR") {
                Style::default().fg(Color::Red)
            } else if m.contains("Clicked") {
                Style::default().fg(Color::Green)
            } else if m.contains("Found button") {
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

    // 3b. All Windows (Right) - Discovery List
    let target_lower = app.config.target_window_title.to_lowercase();
    let all_window_items: Vec<ListItem> = app
        .all_windows
        .iter()
        .map(|w| {
            // Highlight if it matches target config (so user knows what's being picked up)
            if w.to_lowercase().contains(&target_lower) {
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

    let all_windows_list = List::new(all_window_items).block(
        Block::default()
            .borders(Borders::ALL)
            .title("All Visible Windows (Discovery)"),
    );

    frame.render_widget(all_windows_list, middle_chunks[1]);

    // 3. Context Pane (Bottom Split)
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
        frame.render_widget(p, chunks[4]);
    } else {
        let p =
            Paragraph::new("No context data yet. Waiting for button match...").block(context_block);
        frame.render_widget(p, chunks[4]);
    }

    // 4. Footer
    let help = Paragraph::new("Press 'q' or 'Esc' to quit.")
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));

    frame.render_widget(help, chunks[5]);
}
