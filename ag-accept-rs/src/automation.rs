use crate::config::AppConfig;
use crate::services::query::QueryService;
use crate::services::window::WindowService;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;
use tracing::{debug, error, info};
use uiautomation::patterns::UIInvokePattern;
use uiautomation::Result;

pub enum Event {
    Log(String),
    Status(String),
    VisibleWindows(Vec<String>),
    ContextData {
        button: String,
        neighbors: Vec<String>,
    },
    ProcessingWindow(String), // The window currently being checked (cursor)
}

pub struct Automation {
    window_service: WindowService,
    query_service: QueryService,
    config: AppConfig,
    sender: Option<Sender<Event>>,
}

impl Automation {
    pub fn new(config: AppConfig, sender: Option<Sender<Event>>) -> Result<Self> {
        Ok(Self {
            window_service: WindowService::new()?,
            query_service: QueryService::new()?,
            config,
            sender,
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

    pub fn run(&mut self) -> Result<()> {
        self.log("Starting Automation Loop...".to_string());
        let interval = Duration::from_secs_f64(self.config.interval);

        loop {
            self.status("Scanning...".to_string());
            if let Err(_e) = self.cycle() {
                // debug!("Cycle error: {}", e);
            }
            thread::sleep(interval);
        }
    }

    fn cycle(&mut self) -> Result<()> {
        // ALWAYS update visible windows for analyzability
        let target_title = &self.config.target_window_title;
        let target_lower = target_title.to_lowercase();

        if let Ok(titles) = self.window_service.get_all_window_titles() {
            let filtered: Vec<String> = titles
                .into_iter()
                .filter(|t| t.to_lowercase().contains(&target_lower))
                .collect();

            if let Some(tx) = &self.sender {
                let _ = tx.send(Event::VisibleWindows(filtered));
            }
        }

        let target_title = &self.config.target_window_title;

        if let Ok(windows) = self
            .window_service
            .find_all_windows_by_title(target_title, &[])
        {
            if windows.is_empty() {
                self.status(format!(
                    "Target '{}' not found. Scanning visible...",
                    target_title
                ));
            }

            for window in windows {
                let win_name = window.get_name().unwrap_or_default();

                // 1. Notify UI which window is being processed (Cursor)
                if let Some(tx) = &self.sender {
                    let _ = tx.send(Event::ProcessingWindow(win_name.clone()));
                }

                // Wait 1s as requested
                self.status(format!("Processing '{}'...", win_name));
                thread::sleep(Duration::from_secs(1));

                // Optimized Single-Pass Scan
                self.status("Scanning Window...".to_string());
                let scan_result = self.query_service.scan_for_context_and_button(
                    &window,
                    &self.config.context_text_agent_manager,
                    &self.config.search_texts_agent_manager,
                );

                if let Ok((ctx_found, btn_found)) = scan_result {
                    // 1. Check Context
                    if !self.config.context_text_agent_manager.is_empty() {
                        if !ctx_found {
                            self.status("Context mismatch".to_string());
                            continue;
                        }
                        debug!("Context matched");
                    }

                    // 2. Check Button
                    if let Some(button) = btn_found {
                        let btn_name = button.get_name().unwrap_or_default();
                        self.log(format!("Found button: '{}' in '{}'", btn_name, win_name));

                        // Send Context Data
                        if let Ok(siblings) = self.query_service.inspect_siblings(&button) {
                            if let Some(tx) = &self.sender {
                                let _ = tx.send(Event::ContextData {
                                    button: btn_name.clone(),
                                    neighbors: siblings,
                                });
                            }
                        }

                        // Focus window first
                        let _ = self.window_service.focus_window(&window);

                        // Try Invoke
                        let mut success = false;
                        if let Ok(pattern) = button.get_pattern::<UIInvokePattern>() {
                            if let Ok(_) = pattern.invoke() {
                                self.log(format!("Clicked '{}' (Invoke)", btn_name));
                                success = true;
                            }
                        }

                        // Fallback: Click
                        if !success {
                            if let Ok(_) = button.click() {
                                self.log(format!("Clicked '{}' (Click)", btn_name));
                                success = true;
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

                        let _ = self.window_service.restore_previous_focus();
                    } else {
                        self.status("Button not found".to_string());
                    }
                } // End if scan_result
            } // End for loop
        } else {
            self.status(format!(
                "Target '{}' not found. Scanning visible...",
                target_title
            ));
        }

        Ok(())
    }
}
