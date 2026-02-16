use anyhow::Result;

#[derive(Clone, Copy, Debug)]
pub enum Scope {
    #[allow(dead_code)]
    Children,
    Descendants,
}

pub trait Element: Clone + Send + Sync + std::fmt::Debug {
    fn get_name(&self) -> Result<String>;
    fn get_control_type(&self) -> Result<String>;
    fn click(&self) -> Result<()>;
    fn invoke(&self) -> Result<()>;
    fn set_focus(&self) -> Result<()>;
    // For debugging/logging
    fn get_clickable_point(&self) -> Result<(i32, i32)>;
    
    fn find_elements(&self, scope: Scope) -> Result<Vec<Self>>;
}

pub trait Backend: Send + Sync {
    type Element: Element;

    fn new() -> Result<Self> where Self: Sized;
    fn get_root_element(&self) -> Result<Self::Element>;
    fn get_focused_element(&self) -> Result<Self::Element>;
    fn get_all_windows(&self) -> Result<Vec<Self::Element>>;
    
    // Tree traversal abstractions
    fn get_parent(&self, element: &Self::Element) -> Result<Self::Element>;
    #[allow(dead_code)]
    fn get_children(&self, element: &Self::Element) -> Result<Vec<Self::Element>>;
    fn get_siblings(&self, element: &Self::Element) -> Result<(Vec<Self::Element>, Vec<Self::Element>)>; // (prev, next)
}

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::{WindowsBackend as PlatformBackend, WindowsElement as PlatformElement};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::{LinuxBackend as PlatformBackend, LinuxElement as PlatformElement};
