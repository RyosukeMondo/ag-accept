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

    pub fn get_all_windows(&self) -> Result<Vec<UIElement>> {
        let root = self.get_root()?;
        let walker = self.automation.create_tree_walker()?;
        let mut windows = Vec::new();

        let mut result = walker.get_first_child(&root);
        while let Ok(win) = result {
            windows.push(win.clone());
            result = walker.get_next_sibling(&win);
        }
        Ok(windows)
    }

    pub fn get_focused_window_name(&self) -> Result<String> {
        // 1. Get focused element
        let focused = self.automation.get_focused_element()?;

        // 2. Use TreeWalker to walk up
        let walker = self.automation.create_tree_walker()?;
        let root = self.automation.get_root_element()?;

        let mut current = focused;

        // Safety valve for infinite loop
        for _ in 0..20 {
            if let Ok(parent) = walker.get_parent(&current) {
                // Check if parent is root (Desktop)
                if let Ok(true) = self.automation.compare_elements(&parent, &root) {
                    // Current is the top-level window
                    return current.get_name();
                }
                current = parent;
            } else {
                break;
            }
        }

        // Fallback
        current.get_name()
    }
}
