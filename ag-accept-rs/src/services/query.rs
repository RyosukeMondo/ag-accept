use uiautomation::types::TreeScope;
use uiautomation::Result;
use uiautomation::UIAutomation;
use uiautomation::UIElement;

pub struct QueryService {
    automation: UIAutomation,
}

impl QueryService {
    pub fn new() -> Result<Self> {
        Ok(Self {
            automation: UIAutomation::new()?,
        })
    }

    fn get_element_repr(&self, element: &UIElement) -> String {
        let name = element.get_name().unwrap_or_default();
        if !name.trim().is_empty() {
            return format!("\"{}\"", name);
        } else {
            if let Ok(ctype) = element.get_control_type() {
                return format!("[Type:{:?}]", ctype);
            }
        }
        "<Unknown>".to_string()
    }

    pub fn inspect_siblings(&self, element: &UIElement) -> Result<Vec<String>> {
        let walker = self.automation.create_tree_walker()?;
        let mut siblings = Vec::new();

        // 2 Previous
        let mut prev_list = Vec::new();
        let mut curr = element.clone();
        for _ in 0..2 {
            if let Ok(prev) = walker.get_previous_sibling(&curr) {
                prev_list.push(format!("Prev: {}", self.get_element_repr(&prev)));
                curr = prev;
            } else {
                break;
            }
        }
        prev_list.reverse(); // So oldest is first
        siblings.extend(prev_list);

        // Current
        siblings.push(format!("*MATCH*: {}", self.get_element_repr(element)));

        // 2 Next
        let mut curr = element.clone();
        for _ in 0..2 {
            if let Ok(next) = walker.get_next_sibling(&curr) {
                siblings.push(format!("Next: {}", self.get_element_repr(&next)));
                curr = next;
            } else {
                break;
            }
        }

        Ok(siblings)
    }

    pub fn scan_for_context_and_button(
        &self,
        root: &UIElement,
        context_text: &[String],
        button_text: &[String],
    ) -> Result<(bool, Option<UIElement>)> {
        // Bulk Optimization: Get ALL descendants in one COM call
        let condition = self.automation.create_true_condition()?;
        let elements = root.find_all(TreeScope::Descendants, &condition)?;

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
                                let type_str = format!("{:?}", ctype);
                                if type_str.contains("Button") || type_str.contains("Hyperlink") {
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

    pub fn get_ancestry(&self, element: &UIElement) -> Result<Vec<UIElement>> {
        let walker = self.automation.create_tree_walker()?;
        let mut ancestors = Vec::new();
        let mut curr = element.clone();

        // Walk up max 10 levels to avoid infinite loops if tree is cyclic (rare but safe)
        for _ in 0..10 {
            if let Ok(parent) = walker.get_parent(&curr) {
                ancestors.push(parent.clone());
                curr = parent;
            } else {
                break;
            }
        }
        Ok(ancestors)
    }
}
