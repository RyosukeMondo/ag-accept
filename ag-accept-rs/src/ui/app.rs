use crate::automation::WindowStat;
use crate::config::AppConfig;

pub struct App {
    pub logs: Vec<String>,
    pub status: String,
    pub config: AppConfig,
    pub should_quit: bool,
    pub visible_windows: Vec<WindowStat>,
    pub all_windows: Vec<String>,
    pub context_data: Option<(String, Vec<String>)>, // button_name, neighbors
    pub processing_window: Option<String>,
    pub last_scan_ms: u64,
    pub sleep_interval: f64,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let sleep_interval = config.interval;
        Self {
            logs: vec!["Welcome to Ag-Accept TUI".to_string()],
            status: "Idle".to_string(),
            config,
            should_quit: false,
            visible_windows: Vec::new(),
            all_windows: Vec::new(),
            context_data: None,
            processing_window: None,
            last_scan_ms: 0,
            sleep_interval,
        }
    }

    pub fn on_visible_windows(&mut self, windows: Vec<WindowStat>) {
        self.visible_windows = windows;
    }

    pub fn on_all_windows(&mut self, windows: Vec<String>) {
        self.all_windows = windows;
    }

    pub fn on_log(&mut self, message: String) {
        self.logs.push(message);
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    pub fn on_status(&mut self, status: String) {
        self.status = status;
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn on_context(&mut self, button: String, neighbors: Vec<String>) {
        self.context_data = Some((button, neighbors));
    }

    pub fn on_processing(&mut self, window_title: String) {
        self.processing_window = Some(window_title);
    }

    pub fn on_timing(&mut self, scan_ms: u64) {
        self.last_scan_ms = scan_ms;
    }
}
