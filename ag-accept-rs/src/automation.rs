use crate::config::AppConfig;
use crate::services::query::QueryService;
use crate::services::window::WindowService;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::{Duration, Instant};
use tracing::{error, info}; // removed unused debug
use uiautomation::patterns::UIInvokePattern;
use uiautomation::Result;
use uiautomation::UIElement;

#[derive(Clone, Debug)]
pub struct WindowStat {
    pub title: String,
    pub duration_ms: u64,
    pub is_target: bool,
    pub is_focused: bool,
}

pub enum Event {
    Log(String),
    Status(String),
    VisibleWindows(Vec<WindowStat>),
    AllWindows(Vec<String>),
    ContextData {
        button: String,
        neighbors: Vec<String>,
    },
    ProcessingWindow(String), // The window currently being checked (cursor)
    Timing(u64),              // Last scan duration in ms
}

pub struct Automation {
    window_service: WindowService,
    query_service: QueryService,
    config: AppConfig,
    sender: Option<Sender<Event>>,
    cached_button: Option<UIElement>,
    cached_ancestry: Vec<UIElement>, // Layered Cache: Parent -> Grandparent -> ...
    last_durations: HashMap<String, u64>,
}

impl Automation {
    pub fn new(config: AppConfig, sender: Option<Sender<Event>>) -> Result<Self> {
        Ok(Self {
            window_service: WindowService::new()?,
            query_service: QueryService::new()?,
            config,
            sender,
            cached_button: None,
            cached_ancestry: Vec::new(),
            last_durations: HashMap::new(),
        })
    }

    fn log(&self, msg: String) {
        info!("{}", msg);
        if let Some(tx) = &self.sender {
            let _ = tx.send(Event::Log(msg));
        }
    }

    fn status(&self, msg: String) {
        if let Some(tx) = &self.sender {
            let _ = tx.send(Event::Status(msg));
        }
    }

    fn publish_snapshot(&self, mut stats: Vec<WindowStat>) {
        if let Some(tx) = &self.sender {
            // Stability: Always sort by title
            stats.sort_by(|a, b| a.title.cmp(&b.title));
            let _ = tx.send(Event::VisibleWindows(stats));
        }
    }

    pub fn run(&mut self) -> Result<()> {
        self.log("Starting Automation Loop...".to_string());
        let interval = Duration::from_secs_f64(self.config.interval);

        loop {
            // 1. Measure Scan
            let start = Instant::now();
            let mut high_load = false;

            match self.cycle() {
                Ok(hl) => high_load = hl,
                Err(_e) => {}
            }

            let duration = start.elapsed();

            // 2. Report Timing
            if let Some(tx) = &self.sender {
                let _ = tx.send(Event::Timing(duration.as_millis() as u64));
            }

            // 3. Adaptive Sleep
            let sleep_duration = if high_load {
                let backoff = self.config.interval * 2.0;
                let backoff = if backoff < 2.0 { 2.0 } else { backoff }; // Min 2s backoff
                self.status(format!("Backoff (High Load) - Sleeping {:.1}s...", backoff));
                Duration::from_secs_f64(backoff)
            } else {
                self.status(format!("Sleeping ({:.1}s)...", self.config.interval));
                interval
            };

            thread::sleep(sleep_duration);
        }
    }

    fn cycle(&mut self) -> Result<bool> {
        let target_title = &self.config.target_window_title;
        let target_lower = target_title.to_lowercase();
        let focused_name = self
            .window_service
            .get_focused_window_name()
            .unwrap_or_default();
        let focused_lower = focused_name.to_lowercase();

        // 1. SINGLE SOURCE OF TRUTH: Get all windows once
        let all_windows = self.window_service.get_all_windows().unwrap_or_default();
        let mut titles = Vec::new();
        let mut target_windows = Vec::new();

        // Define exclusions
        let exclusions = vec![
            "ag-accept".to_string(),
            "cmd.exe".to_string(),
            "powershell".to_string(),
            "Windows PowerShell".to_string(),
        ];

        // 2. Classify Windows (Discovery vs Target)
        // We filter manually here to avoid re-fetching names
        for window in all_windows {
            if let Ok(name) = window.get_name() {
                if !name.trim().is_empty() {
                    titles.push(name.clone());

                    if name.to_lowercase().contains(&target_lower) {
                        // Apply Exclusions HERE
                        if !exclusions.iter().any(|ex| name.contains(ex)) {
                            target_windows.push((name, window));
                        }
                    }
                }
            }
        }

        // 3. Broadcast Discovery List
        titles.sort();
        if let Some(tx) = &self.sender {
            let _ = tx.send(Event::AllWindows(titles));
        }

        // 4. Phase 1 Snapshot (Memory)
        // Construct stable stats from memory before doing work
        let current_stats: Vec<WindowStat> = target_windows
            .iter()
            .map(|(name, _)| {
                let dur = *self.last_durations.get(name).unwrap_or(&0);
                WindowStat {
                    title: name.clone(),
                    duration_ms: dur,
                    is_target: false,
                    is_focused: !focused_lower.is_empty() && name.to_lowercase() == focused_lower,
                }
            })
            .collect();
        self.publish_snapshot(current_stats);

        // --- TIER 1 Checking (Cached Button) ---
        if let Some(button) = &self.cached_button {
            if let Ok(name) = button.get_name() {
                if !name.trim().is_empty() {
                    self.log(format!("Using Cached Button: '{}' (Instant Scan)", name));
                    self.perform_action(button.clone(), "Cached Window".to_string());
                    return Ok(false);
                }
            }
            self.cached_button = None;
        }

        // --- TIER 2 Checking (Ancestry) ---
        // (Optimized: Checking cached ancestry before full scan)
        // ... (Ancestry logic omitted for brevity, keeping simple for this refactor to focus on stability)
        // Actually we should keep ancestry logic. It's safe to keep.

        let ancestors = self.cached_ancestry.clone();
        for ancestor in ancestors.iter() {
            if let Ok(_) = ancestor.get_name() {
                if let Ok((ctx, btn_opt)) = self.query_service.scan_for_context_and_button(
                    ancestor,
                    &self.config.context_text_agent_manager,
                    &self.config.search_texts_agent_manager,
                ) {
                    if ctx && btn_opt.is_some() {
                        let button = btn_opt.unwrap();
                        self.cached_button = Some(button.clone());
                        if let Ok(new_ancestry) = self.query_service.get_ancestry(&button) {
                            self.cached_ancestry = new_ancestry;
                        }
                        self.perform_action(button, "Cached Ancestor".to_string());
                        // Snapshot is already valid (old times), returning early is fine.
                        return Ok(false);
                    }
                }
            }
        }

        // 5. Phase 3: Full Scan (Processing Targets)
        let mut window_stats = Vec::new();
        let mut high_load_detected = false;

        if target_windows.is_empty() {
            self.status(format!("Target '{}' not found.", target_title));
        }

        for (win_name, window) in target_windows {
            // Exclusions already applied in Step 2

            // MEASUREMENT
            let win_start = Instant::now();
            if let Some(tx) = &self.sender {
                let _ = tx.send(Event::ProcessingWindow(win_name.clone()));
            }

            thread::sleep(Duration::from_millis(50));

            let scan_result = self.query_service.scan_for_context_and_button(
                &window,
                &self.config.context_text_agent_manager,
                &self.config.search_texts_agent_manager,
            );

            let win_duration = win_start.elapsed();
            let win_ms = win_duration.as_millis() as u64;

            // Update Memory
            self.last_durations.insert(win_name.clone(), win_ms);

            // Add to fresh stats list
            window_stats.push(WindowStat {
                title: win_name.clone(),
                duration_ms: win_ms,
                is_target: true,
                is_focused: !focused_lower.is_empty() && win_name.to_lowercase() == focused_lower,
            });

            if win_ms > 1000 {
                high_load_detected = true;
                self.log(format!("High Load: '{}' took {}ms", win_name, win_ms));
            }

            // Logic Handling (Context/Button) - Same as before
            if let Ok((ctx_found, btn_found)) = scan_result {
                if !self.config.context_text_agent_manager.is_empty() && !ctx_found {
                    continue;
                }
                if let Some(button) = btn_found {
                    let btn_name = button.get_name().unwrap_or_default();
                    self.log(format!("Found button: '{}' in '{}'", btn_name, win_name));

                    self.cached_button = Some(button.clone());
                    if let Ok(new_ancestry) = self.query_service.get_ancestry(&button) {
                        self.cached_ancestry = new_ancestry;
                    }
                    if let Ok(siblings) = self.query_service.inspect_siblings(&button) {
                        if let Some(tx) = &self.sender {
                            let _ = tx.send(Event::ContextData {
                                button: btn_name,
                                neighbors: siblings,
                            });
                        }
                    }

                    let _ = self.window_service.focus_window(&window);
                    self.perform_action(button, win_name);
                    let _ = self.window_service.restore_previous_focus();
                }
            }
        }

        // 6. Broadcast Updated Snapshot
        self.publish_snapshot(window_stats);

        Ok(high_load_detected)
    }

    fn perform_action(&self, button: UIElement, _win_name: String) {
        let btn_name = button.get_name().unwrap_or_default();
        let mut success = false;

        // Try Invoke
        if let Ok(pattern) = button.get_pattern::<UIInvokePattern>() {
            if let Ok(_) = pattern.invoke() {
                self.log(format!("Clicked '{}' (Invoke)", btn_name));
                success = true;
            }
        }

        // Fallback: Click
        if !success {
            // Validate coordinates first
            if let Ok(pt) = button.get_clickable_point() {
                // Use debug formatting to avoid private field access issues
                let pt_str = format!("{:?}", pt);
                if pt_str.contains("x: 0") && pt_str.contains("y: 0") {
                    self.log(format!(
                        "Skipping click: Invalid coordinates (0,0) for '{}'",
                        btn_name
                    ));
                } else {
                    if let Ok(_) = button.click() {
                        self.log(format!("Clicked '{}' (Click) at {:?}", btn_name, pt));
                        success = true;
                    }
                }
            } else {
                self.log(format!(
                    "Skipping click: No clickable point for '{}'",
                    btn_name
                ));
            }
        }

        if !success {
            let err = "Failed to click button via Invoke or Click".to_string();
            error!("{}", err);
            self.log(format!("ERROR: {}", err));
            self.status("Action Failed".to_string());
        } else {
            self.status("Success!".to_string());
            thread::sleep(Duration::from_millis(500));
        }
    }
}
