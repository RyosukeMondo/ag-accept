use crate::platform::{Backend, PlatformBackend, PlatformElement, Scope};
use anyhow::Result;

pub struct QueryService {
    backend: PlatformBackend,
}

impl QueryService {
    pub fn new() -> Result<Self> {
        Ok(Self {
            backend: PlatformBackend::new()?,
        })
    }

    fn get_element_repr(&self, element: &PlatformElement) -> String {
        use crate::platform::Element;
        let name = element.get_name().unwrap_or_default();
        if !name.trim().is_empty() {
            return format!("\"{}\"", name);
        } else {
            if let Ok(ctype) = element.get_control_type() {
                return format!("[Type:{}]", ctype);
            }
        }
        "<Unknown>".to_string()
    }

    pub fn inspect_siblings(&self, element: &PlatformElement) -> Result<Vec<String>> {
        let mut siblings = Vec::new();
        let (prev_list, next_list) = self.backend.get_siblings(element)?;

        for prev in prev_list {
            siblings.push(format!("Prev: {}", self.get_element_repr(&prev)));
        }
        
        // Current
        siblings.push(format!("*MATCH*: {}", self.get_element_repr(element)));

        for next in next_list {
            siblings.push(format!("Next: {}", self.get_element_repr(&next)));
        }

        Ok(siblings)
    }

    pub fn scan_for_context_and_button(
        &self,
        root: &PlatformElement,
        context_text: &[String],
        button_text: &[String],
    ) -> Result<(bool, Option<PlatformElement>)> {
        use crate::platform::Element;
        // Bulk Optimization: Get ALL descendants in one COM call (on Windows)
        let elements = root.find_elements(Scope::Descendants)?;

        let mut ctx_found = false;
        let mut btn_found = None;

        for element in elements {
            if let Ok(name) = element.get_name() {
                if name.trim().is_empty() {
                    continue;
                }

                // Check Context
                if !context_text.is_empty() && !ctx_found {
                    for part in context_text {
                        if name.contains(part) {
                            ctx_found = true;
                            // Don't break, need button
                        }
                    }
                }

                // Check Button with Safe String Type Check
                if !button_text.is_empty() && btn_found.is_none() {
                    for part in button_text {
                        if name.contains(part) {
                            if let Ok(ctype) = element.get_control_type() {
                                // ctype is already a String in our abstraction
                                if ctype.contains("Button") || ctype.contains("Hyperlink") {
                                    btn_found = Some(element.clone());
                                }
                            }
                        }
                    }
                }

                if ctx_found && btn_found.is_some() {
                    return Ok((true, btn_found));
                }
            }
        }

        Ok((ctx_found, btn_found))
    }

    pub fn get_ancestry(&self, element: &PlatformElement) -> Result<Vec<PlatformElement>> {
        let mut ancestors = Vec::new();
        let mut curr = element.clone();

        // Walk up max 10 levels to avoid infinite loops if tree is cyclic (rare but safe)
        for _ in 0..10 {
            if let Ok(parent) = self.backend.get_parent(&curr) {
                ancestors.push(parent.clone());
                curr = parent;
            } else {
                break;
            }
        }
        Ok(ancestors)
    }
}
