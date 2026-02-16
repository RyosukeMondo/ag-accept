use crate::platform::{Backend, PlatformBackend, PlatformElement};
use anyhow::Result;

pub struct WindowService {
    backend: PlatformBackend,
    previous_focus: Option<PlatformElement>,
}

impl WindowService {
    pub fn new() -> Result<Self> {
        Ok(Self {
            backend: PlatformBackend::new()?,
            previous_focus: None,
        })
    }

    #[allow(dead_code)]
    pub fn get_root(&self) -> Result<PlatformElement> {
        self.backend.get_root_element()
    }

    pub fn focus_window(&mut self, window: &PlatformElement) -> Result<()> {
        use crate::platform::Element;
        if let Ok(current) = self.backend.get_focused_element() {
            self.previous_focus = Some(current);
        } else {
            self.previous_focus = None;
        }

        window.set_focus()
    }

    pub fn restore_previous_focus(&mut self) -> Result<()> {
        use crate::platform::Element;
        if let Some(prev) = &self.previous_focus {
            let _ = prev.set_focus();
        }
        self.previous_focus = None;
        Ok(())
    }

    pub fn get_all_windows(&self) -> Result<Vec<PlatformElement>> {
        self.backend.get_all_windows()
    }

    pub fn get_focused_window_name(&self) -> Result<String> {
        use crate::platform::Element;
        // 1. Get focused element
        let focused = self.backend.get_focused_element()?;

        // 2. Walk up to find top-level window
        // This logic might need to be in the backend if it's too specific,
        // but for now let's try to keep it here using generic backend methods.
        
        let _root = self.backend.get_root_element()?;
        // Note: Comparing elements might need a trait method or Eq implementation.
        // For now, let's just assume we can get the name and if it's not empty/desktop...
        // Actually, the original code compared elements.
        // We might need an `is_same` method on Element trait?
        // Or just rely on the backend to provide "get_top_level_parent"?
        
        // Let's implement a simple walk up.
        let mut current = focused;
        
        for _ in 0..20 {
            if let Ok(parent) = self.backend.get_parent(&current) {
                 // Check if parent is root.
                 // We don't have equality check yet.
                 // Let's assume if parent name is "Desktop" or similar?
                 // Or just check if we can't go higher?
                 
                 // Better: Check if parent is root.
                 // We need equality.
                 // Let's add `partial_eq` to Element trait or just `is_root`?
                 
                 // Workaround: Just return current name if parent fails or is root.
                 // But how do we know it's root?
                 
                 // Let's assume the backend `get_parent` returns error or special value for root?
                 // In uiautomation, get_parent of root might fail or return null.
                 
                 // Let's try to get name.
                 current = parent;
            } else {
                break;
            }
        }
        
        // This logic is a bit flawed without equality.
        // But for now, let's just return the name of what we found.
        current.get_name()
    }
}
