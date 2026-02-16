use super::{Backend, Element};
use anyhow::{anyhow, Result};
use uiautomation::types::TreeScope;
use uiautomation::{UIAutomation, UIElement};

#[derive(Clone, Debug)]
pub struct WindowsElement(UIElement);

impl Element for WindowsElement {
    fn get_name(&self) -> Result<String> {
        self.0.get_name().map_err(|e| anyhow!(e))
    }

    fn get_control_type(&self) -> Result<String> {
        self.0.get_control_type().map(|t| format!("{:?}", t)).map_err(|e| anyhow!(e))
    }

    fn click(&self) -> Result<()> {
        self.0.click().map_err(|e| anyhow!(e))
    }

    fn invoke(&self) -> Result<()> {
        use uiautomation::patterns::UIInvokePattern;
        if let Ok(pattern) = self.0.get_pattern::<UIInvokePattern>() {
            pattern.invoke().map_err(|e| anyhow!(e))
        } else {
            Err(anyhow!("Invoke pattern not supported"))
        }
    }

    fn set_focus(&self) -> Result<()> {
        self.0.set_focus().map_err(|e| anyhow!(e))
    }

    fn get_clickable_point(&self) -> Result<(i32, i32)> {
        self.0.get_clickable_point().map(|p| (p.x, p.y)).map_err(|e| anyhow!(e))
    }

    fn find_elements(&self, scope: super::Scope) -> Result<Vec<Self>> {
        let condition = self.0.create_true_condition().map_err(|e| anyhow!(e))?;
        let tree_scope = match scope {
            super::Scope::Children => TreeScope::Children,
            super::Scope::Descendants => TreeScope::Descendants,
        };
        let elements = self.0.find_all(tree_scope, &condition).map_err(|e| anyhow!(e))?;
        Ok(elements.into_iter().map(WindowsElement).collect())
    }
}

pub struct WindowsBackend {
    automation: UIAutomation,
}

impl Backend for WindowsBackend {
    type Element = WindowsElement;

    fn new() -> Result<Self> {
        Ok(Self {
            automation: UIAutomation::new().map_err(|e| anyhow!(e))?,
        })
    }

    fn get_root_element(&self) -> Result<Self::Element> {
        self.automation.get_root_element().map(WindowsElement).map_err(|e| anyhow!(e))
    }

    fn get_focused_element(&self) -> Result<Self::Element> {
        self.automation.get_focused_element().map(WindowsElement).map_err(|e| anyhow!(e))
    }

    fn get_all_windows(&self) -> Result<Vec<Self::Element>> {
        let root = self.automation.get_root_element().map_err(|e| anyhow!(e))?;
        let walker = self.automation.create_tree_walker().map_err(|e| anyhow!(e))?;
        let mut windows = Vec::new();

        let mut result = walker.get_first_child(&root);
        while let Ok(win) = result {
            windows.push(WindowsElement(win.clone()));
            result = walker.get_next_sibling(&win);
        }
        Ok(windows)
    }

    fn get_parent(&self, element: &Self::Element) -> Result<Self::Element> {
        let walker = self.automation.create_tree_walker().map_err(|e| anyhow!(e))?;
        walker.get_parent(&element.0).map(WindowsElement).map_err(|e| anyhow!(e))
    }

    fn get_children(&self, element: &Self::Element) -> Result<Vec<Self::Element>> {
        // Optimization: Use FindAll for descendants if needed, but for direct children walker is safer?
        // Actually, FindAll with TreeScope::Children is better.
        let condition = self.automation.create_true_condition().map_err(|e| anyhow!(e))?;
        let children = element.0.find_all(TreeScope::Children, &condition).map_err(|e| anyhow!(e))?;
        Ok(children.into_iter().map(WindowsElement).collect())
    }

    fn get_siblings(&self, element: &Self::Element) -> Result<(Vec<Self::Element>, Vec<Self::Element>)> {
        let walker = self.automation.create_tree_walker().map_err(|e| anyhow!(e))?;
        
        let mut prev_list = Vec::new();
        let mut curr = element.0.clone();
        for _ in 0..2 {
            if let Ok(prev) = walker.get_previous_sibling(&curr) {
                prev_list.push(WindowsElement(prev.clone()));
                curr = prev;
            } else {
                break;
            }
        }
        prev_list.reverse();

        let mut next_list = Vec::new();
        let mut curr = element.0.clone();
        for _ in 0..2 {
            if let Ok(next) = walker.get_next_sibling(&curr) {
                next_list.push(WindowsElement(next.clone()));
                curr = next;
            } else {
                break;
            }
        }

        Ok((prev_list, next_list))
    }
}
