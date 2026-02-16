use super::{Backend, Element};
use anyhow::{anyhow, Result};
use std::sync::Arc;
use std::fmt;
use tracing::{info, error, debug};

// Try atspi::proxy::accessible::AccessibleProxy
use atspi::proxy::accessible::AccessibleProxy; 

#[derive(Clone)]
pub struct LinuxElement {
    connection: Arc<atspi::connection::AccessibilityConnection>,
    runtime: Arc<tokio::runtime::Runtime>,
    bus_name: String,
    path: String,
}

impl fmt::Debug for LinuxElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LinuxElement")
            .field("bus_name", &self.bus_name)
            .field("path", &self.path)
            .finish()
    }
}

impl Element for LinuxElement {
    fn get_name(&self) -> Result<String> {
        let connection = self.connection.clone();
        let bus_name = self.bus_name.clone();
        let path = self.path.clone();

        let name = self.runtime.block_on(async move {
                let proxy = AccessibleProxy::builder((**connection).connection())
                    .destination(bus_name)?
                    .path(path)?
                    .build()
                    .await?;
                proxy.name().await
            })
            .map_err(|e| anyhow!("Failed to get name: {:?}", e))?;
        Ok(name)
    }

    fn get_control_type(&self) -> Result<String> {
        let connection = self.connection.clone();
        let bus_name = self.bus_name.clone();
        let path = self.path.clone();

        let role = self.runtime.block_on(async move {
                let proxy = AccessibleProxy::builder((**connection).connection())
                    .destination(bus_name)?
                    .path(path)?
                    .build()
                    .await?;
                proxy.get_role().await
            })
            .map_err(|e| anyhow!("Failed to get role: {:?}", e))?;
        Ok(format!("{:?}", role))
    }

    fn click(&self) -> Result<()> {
        Err(anyhow!("Click not implemented"))
    }

    fn invoke(&self) -> Result<()> {
        Err(anyhow!("Invoke not implemented"))
    }

    fn set_focus(&self) -> Result<()> {
        Err(anyhow!("SetFocus not implemented"))
    }

    fn get_clickable_point(&self) -> Result<(i32, i32)> {
        Err(anyhow!("GetClickablePoint not implemented"))
    }

    fn find_elements(&self, _scope: super::Scope) -> Result<Vec<Self>> {
        Err(anyhow!("FindElements not implemented"))
    }
}

pub struct LinuxBackend {
    connection: Arc<atspi::connection::AccessibilityConnection>,
    runtime: Arc<tokio::runtime::Runtime>,
}

impl Backend for LinuxBackend {
    type Element = LinuxElement;

    fn new() -> Result<Self> {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let connection = runtime.block_on(atspi::connection::AccessibilityConnection::open())
            .map_err(|e| anyhow!("Failed to open connection: {:?}", e))?;
        
        Ok(Self { 
            connection: Arc::new(connection),
            runtime: Arc::new(runtime),
        })
    }

    fn get_root_element(&self) -> Result<Self::Element> {
        // Root is at /org/a11y/atspi/accessible/root on org.a11y.atspi.Registry
        Ok(LinuxElement {
            connection: self.connection.clone(),
            runtime: self.runtime.clone(),
            bus_name: "org.a11y.atspi.Registry".to_string(),
            path: "/org/a11y/atspi/accessible/root".to_string(),
        })
    }

    fn get_focused_element(&self) -> Result<Self::Element> {
        Err(anyhow!("GetFocused not implemented"))
    }

    fn get_all_windows(&self) -> Result<Vec<Self::Element>> {
        // To get all windows, we usually iterate children of the root (applications)
        // and then children of applications (windows).
        debug!("Getting root element...");
        let root = self.get_root_element()?;
        debug!("Getting children of root (apps)...");
        let apps = match self.get_children(&root) {
            Ok(a) => a,
            Err(e) => {
                error!("Failed to get root children: {:?}", e);
                return Err(e);
            }
        };
        info!("Found {} apps/children under root", apps.len());
        
        let mut windows = Vec::new();
        for app in apps {
            // Some apps might fail to return children, ignore them
            // debug!("Getting children for app {:?}", app); // app debug might be verbose
            match self.get_children(&app) {
                Ok(app_windows) => {
                    // debug!("App has {} windows", app_windows.len());
                    windows.extend(app_windows);
                },
                Err(e) => {
                    // This is common for some registry entries that aren't real apps
                    debug!("Failed to get children for app: {:?}", e);
                }
            }
        }
        info!("Found total {} windows across all apps", windows.len());
        Ok(windows)
    }

    fn get_parent(&self, _element: &Self::Element) -> Result<Self::Element> {
        Err(anyhow!("GetParent not implemented"))
    }

    fn get_children(&self, element: &Self::Element) -> Result<Vec<Self::Element>> {
        let connection = self.connection.clone();
        let bus_name = element.bus_name.clone();
        let path = element.path.clone();

        let children_data = self.runtime.block_on(async move {
                let conn = (**connection).connection();
                let proxy: zbus::Proxy = zbus::ProxyBuilder::new_bare(conn)
                    .destination(bus_name.clone())?
                    .path(path.clone())?
                    .interface("org.a11y.atspi.Accessible")?
                    .cache_properties(zbus::CacheProperties::No)
                    .build()
                    .await.map_err(|e| anyhow!("Failed to build proxy: {:?}", e))?;

                let count: i32 = proxy.get_property("ChildCount").await
                    .map_err(|e| anyhow!("Failed to get ChildCount: {:?}", e))?;
                
                let mut results = Vec::new();
                for i in 0..count {
                    let (child_bus, child_path): (String, zbus::zvariant::OwnedObjectPath) = proxy.call(
                        "GetChildAtIndex",
                        &(i)
                    ).await.map_err(|e| anyhow!("Failed to GetChildAtIndex({}): {:?}", i, e))?;
                    
                    results.push((child_bus, child_path.to_string()));
                }
                Ok::<_, anyhow::Error>(results)
            })
            .map_err(|e| anyhow!("Failed to get children for {} {}: {:?}", element.bus_name, element.path, e))?;

        let mut result = Vec::new();
        for (c_bus, c_path) in children_data {
             result.push(LinuxElement {
                 connection: self.connection.clone(),
                 runtime: self.runtime.clone(),
                 bus_name: c_bus,
                 path: c_path,
             });
        }
        
        Ok(result)
    }

    fn get_siblings(&self, _element: &Self::Element) -> Result<(Vec<Self::Element>, Vec<Self::Element>)> {
        Err(anyhow!("GetSiblings not implemented"))
    }
}