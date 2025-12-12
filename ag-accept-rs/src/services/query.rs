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

    pub fn find_button_with_text(
        &self,
        root: &UIElement,
        text_parts: &[String],
    ) -> Result<Option<UIElement>> {
        // Recursive walk to find button with text.
        // uiautomation walker is best.
        let walker = self.automation.create_tree_walker()?;

        // We can do a BFS or DFS. `has_text_recursive` internal logic might be reusable if generic?
        // But here we want to return the element, specifically a button.
        // We'll iterate and check ControlType string name?
        // Or just match loose logic: "Has text X and looks like button?"
        // Usually safer to just check `get_name().contains(X)` and maybe `get_control_type()`?
        // But control type enum is elusive. `control_type_name`?

        // Let's implement recursive search for element that has name containing text.
        // We won't filter by ControlType Button strictly to avoid import issues, unless we check string "Button".

        self.find_element_recursive_internal(root, &walker, text_parts, 0, 25)
    }

    fn find_element_recursive_internal(
        &self,
        element: &UIElement,
        walker: &uiautomation::UITreeWalker,
        text_parts: &[String],
        depth: usize,
        max_depth: usize,
    ) -> Result<Option<UIElement>> {
        if depth > max_depth {
            return Ok(None);
        }

        if let Ok(name) = element.get_name() {
            for part in text_parts {
                if name.contains(part) {
                    // Check if it's a button?
                    // element.get_control_type() returns ID.
                    // But we don't have constants.
                    // We'll return it. The "Accept" text usually belongs to the button itself or a label inside.
                    // If it's a label inside button, Invoke might not work on label.
                    // But generally uiautomation finds the control.
                    // Let's assume hitting "Accept" text element is actionable or its parent is.
                    return Ok(Some(element.clone()));
                }
            }
        }

        if let Ok(mut child) = walker.get_first_child(element) {
            loop {
                if let Some(found) = self.find_element_recursive_internal(
                    &child,
                    walker,
                    text_parts,
                    depth + 1,
                    max_depth,
                )? {
                    return Ok(Some(found));
                }

                if let Ok(next) = walker.get_next_sibling(&child) {
                    child = next;
                } else {
                    break;
                }
            }
        }

        Ok(None)
    }

    pub fn has_text_recursive(&self, root: &UIElement, text_parts: &[String]) -> bool {
        let Ok(walker) = self.automation.create_tree_walker() else {
            return false;
        };

        match self.has_text_recursive_internal(root, &walker, text_parts, 0, 25) {
            Ok(res) => res,
            Err(_) => false,
        }
    }

    fn has_text_recursive_internal(
        &self,
        element: &UIElement,
        walker: &uiautomation::UITreeWalker,
        text_parts: &[String],
        depth: usize,
        max_depth: usize,
    ) -> Result<bool> {
        if depth > max_depth {
            return Ok(false);
        }

        if let Ok(name) = element.get_name() {
            for part in text_parts {
                if name.contains(part) {
                    return Ok(true);
                }
            }
        }

        if let Ok(mut child) = walker.get_first_child(element) {
            loop {
                if self.has_text_recursive_internal(
                    &child,
                    walker,
                    text_parts,
                    depth + 1,
                    max_depth,
                )? {
                    return Ok(true);
                }
                if let Ok(next) = walker.get_next_sibling(&child) {
                    child = next;
                } else {
                    break;
                }
            }
        }

        Ok(false)
    }

    fn get_element_repr(&self, element: &UIElement) -> String {
        let name = element.get_name().unwrap_or_default();
        if !name.trim().is_empty() {
            // "Accept"
            return format!("\"{}\"", name);
        } else {
            // [Button] or [Pane]
            if let Ok(ctype) = element.get_control_type() {
                // Use Debug formatting for ControlType
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

    // Optimization: Single Pass Scan
    // Returns (ContextFound, ButtonFound)
    pub fn scan_for_context_and_button(
        &self,
        root: &UIElement,
        context_text: &[String],
        button_text: &[String],
    ) -> Result<(bool, Option<UIElement>)> {
        let walker = self.automation.create_tree_walker()?;
        self.scan_recursive(root, &walker, context_text, button_text, 0, 25)
    }

    fn scan_recursive(
        &self,
        element: &UIElement,
        walker: &uiautomation::UITreeWalker,
        context_text: &[String],
        button_text: &[String],
        depth: usize,
        max_depth: usize,
    ) -> Result<(bool, Option<UIElement>)> {
        if depth > max_depth {
            return Ok((false, None));
        }

        let mut ctx_found = false;
        let mut btn_found = None;

        // Check current node
        if let Ok(name) = element.get_name() {
            // Check Context
            if !context_text.is_empty() && !ctx_found {
                for part in context_text {
                    if name.contains(part) {
                        ctx_found = true;
                        break;
                    }
                }
            }

            // Check Button (if not already found)
            if !button_text.is_empty() && btn_found.is_none() {
                for part in button_text {
                    if name.contains(part) {
                        // Improve match by checking if it looks like a button?
                        // For now, simple text match as before.
                        btn_found = Some(element.clone());
                        break;
                    }
                }
            }
        }

        // Short circuit if both found?
        // Logic: specific automation requires Context AND Button.
        // If we found context node, we still search button.
        // If we found button, we still search context.
        // If we found BOTH, we can stop?
        // Actually, context text might be anywhere.
        // But if both are found, we can return.
        if ctx_found && btn_found.is_some() {
            return Ok((true, btn_found));
        }

        // Children
        if let Ok(mut child) = walker.get_first_child(element) {
            loop {
                let (child_ctx, child_btn) = self.scan_recursive(
                    &child,
                    walker,
                    context_text,
                    button_text,
                    depth + 1,
                    max_depth,
                )?;

                if child_ctx {
                    ctx_found = true;
                }
                if child_btn.is_some() && btn_found.is_none() {
                    btn_found = child_btn;
                }

                if ctx_found && btn_found.is_some() {
                    return Ok((true, btn_found));
                }

                if let Ok(next) = walker.get_next_sibling(&child) {
                    child = next;
                } else {
                    break;
                }
            }
        }

        Ok((ctx_found, btn_found))
    }
}
