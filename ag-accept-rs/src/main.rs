mod automation;
mod config;
mod services;
mod ui;

use anyhow::Result;
use automation::{Automation, Event};
use config::AppConfig;
use crossterm::event::{self, Event as CEvent, KeyCode};

use std::sync::mpsc;

use std::thread;
use std::time::Duration;
use ui::app::App;

fn main() -> Result<()> {
    // Setup logging to file since stdout is taken by TUI
    // Setup logging to file
    let file_appender = tracing_appender::rolling::daily(".", "ag-accept.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    let config = AppConfig::load()?;
    let config_clone = config.clone();

    // Channel for communication
    let (tx, rx) = mpsc::channel();

    // Automation Thread
    thread::spawn(move || {
        let mut automation = match Automation::new(config_clone, Some(tx)) {
            Ok(a) => a,
            Err(e) => panic!("Failed to init automation: {}", e),
        };
        if let Err(e) = automation.run() {
            eprintln!("Automation error: {}", e);
        }
    });

    // TUI (Main Thread)
    let mut terminal = ui::tui::init()?;
    let mut app = App::new(config);

    loop {
        terminal.draw(|f| ui::ui::render(&mut app, f))?;

        // Handle inputs
        if event::poll(Duration::from_millis(100))? {
            if let CEvent::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => app.quit(),
                    _ => {}
                }
            }
        }

        // Handle events
        while let Ok(event) = rx.try_recv() {
            match event {
                Event::Log(msg) => app.on_log(msg),
                Event::Status(msg) => app.on_status(msg),
                Event::VisibleWindows(wins) => app.on_visible_windows(wins),
                Event::AllWindows(wins) => app.on_all_windows(wins),
                Event::ContextData { button, neighbors } => app.on_context(button, neighbors),
                Event::ProcessingWindow(win) => app.on_processing(win),
                Event::Timing(ms) => app.on_timing(ms),
            }
        }

        if app.should_quit {
            break;
        }
    }

    ui::tui::restore()?;
    Ok(())
}
