use uiautomation::Result;
use uiautomation::UIAutomation;
use uiautomation::UIElement;

pub struct WindowService {
    automation: UIAutomation,
    previous_focus: Option<UIElement>,
}

impl WindowService {
    pub fn new() -> Result<Self> {
        Ok(Self {
            automation: UIAutomation::new()?,
            previous_focus: None,
        })
    }

    pub fn get_root(&self) -> Result<UIElement> {
        self.automation.get_root_element()
    }

    pub fn find_window_by_title(
        &self,
        title_part: &str,
        exclude_titles: &[String],
    ) -> Result<Option<UIElement>> {
        let root = self.get_root()?;
        let walker = self.automation.create_tree_walker()?;

        // Manual iteration avoiding Result clone issues
        let mut result = walker.get_first_child(&root);
        let title_lower = title_part.to_lowercase();

        while let Ok(win) = result {
            if let Ok(name) = win.get_name() {
                if !self.is_excluded(&name, exclude_titles) {
                    if name.to_lowercase().contains(&title_lower) {
                        return Ok(Some(win));
                    }
                }
            }

            result = walker.get_next_sibling(&win);
        }

        Ok(None)
    }

    pub fn find_all_windows_by_title(
        &self,
        title_part: &str,
        exclude_titles: &[String],
    ) -> Result<Vec<UIElement>> {
        let root = self.get_root()?;
        let walker = self.automation.create_tree_walker()?;
        let mut windows = Vec::new();

        let mut result = walker.get_first_child(&root);
        let title_lower = title_part.to_lowercase();

        while let Ok(win) = result {
            if let Ok(name) = win.get_name() {
                if !self.is_excluded(&name, exclude_titles) {
                    if name.to_lowercase().contains(&title_lower) {
                        windows.push(win.clone());
                    }
                }
            }
            result = walker.get_next_sibling(&win);
        }

        Ok(windows)
    }

    pub fn focus_window(&mut self, window: &UIElement) -> Result<()> {
        if let Ok(current) = self.automation.get_focused_element() {
            self.previous_focus = Some(current);
        } else {
            self.previous_focus = None;
        }

        window.set_focus()
    }

    pub fn restore_previous_focus(&mut self) -> Result<()> {
        if let Some(prev) = &self.previous_focus {
            let _ = prev.set_focus();
        }
        self.previous_focus = None;
        Ok(())
    }

    fn is_excluded(&self, name: &str, exclude_titles: &[String]) -> bool {
        let name_lower = name.to_lowercase();
        for ex in exclude_titles {
            if name_lower.contains(&ex.to_lowercase()) {
                return true;
            }
        }
        false
    }

    pub fn get_all_window_titles(&self) -> Result<Vec<String>> {
        let root = self.get_root()?;
        let walker = self.automation.create_tree_walker()?;
        let mut titles = Vec::new();

        let mut result = walker.get_first_child(&root);
        while let Ok(win) = result {
            if let Ok(name) = win.get_name() {
                if !name.trim().is_empty() {
                    titles.push(name);
                }
            }
            result = walker.get_next_sibling(&win);
        }
        Ok(titles)
    }
}
