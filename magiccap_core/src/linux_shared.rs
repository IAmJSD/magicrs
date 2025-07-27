use crate::{hotkeys::HotkeyWrapper, reload, statics::run_thread};
use muda::MenuEvent;
use once_cell::sync::OnceCell;
use std::sync::RwLock;
use webkit2gtk::{URISchemeRequest, WebContext, WebContextExt, WebView};

// Defines a wrapper to fake something being safe to send.
pub struct FakeSend<T> {
    pub value: T,
}
unsafe impl<T> Send for FakeSend<T> {}
unsafe impl<T> Sync for FakeSend<T> {}

// Defines the structure for a shared application.
pub struct SharedApplication {
    pub context: FakeSend<WebContext>,
    pub protocol_handler: Box<RwLock<Option<&'static dyn Fn(&URISchemeRequest)>>>,
    pub webview: RwLock<Option<FakeSend<WebView>>>,
    pub tray_icon: RwLock<Option<tray_icon::TrayIcon>>,
    pub menu_event: RwLock<Option<&'static dyn Fn(MenuEvent)>>,
    pub hotkey_wrapper: HotkeyWrapper,
}

// Defines the public variable.
static mut SHARED_APPLICATION: OnceCell<&'static mut SharedApplication> = OnceCell::new();

// Defines the shared application object.
pub fn app() -> &'static mut SharedApplication {
    unsafe { SHARED_APPLICATION.get_mut().unwrap() }
}

// Inspired by https://github.com/spacedriveapp/spacedrive/blob/9fe722021206cc6fd8a08c063412e5399a0e2103/apps/desktop/crates/linux/src/env.rs
// but changed to use the latest wgpu version.
fn has_nvidia() -> bool {
	use wgpu::{
		Backends, DeviceType, Instance, InstanceDescriptor, InstanceFlags, BackendOptions, MemoryBudgetThresholds
	};

    let desc = InstanceDescriptor {
		flags: InstanceFlags::empty(),
		backends: Backends::VULKAN | Backends::GL,
        backend_options: BackendOptions::default(),
        memory_budget_thresholds: MemoryBudgetThresholds::default(),
	};
	let instance = Instance::new(&desc);
	for adapter in instance.enumerate_adapters(Backends::all()) {
		let info = adapter.get_info();
		match info.device_type {
			DeviceType::DiscreteGpu | DeviceType::IntegratedGpu | DeviceType::VirtualGpu => {
				// Nvidia PCI id
				if info.vendor == 0x10de {
					return true;
				}
			}
			_ => {}
		}
	}

	false
}

// The main entrypoint for setting up the application.
pub fn application_init() {
    // Call gtk::init.
    gtk::init().unwrap();

    if has_nvidia() {
        // Workaround for: https://github.com/tauri-apps/tauri/issues/9304
		std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }

    // Handle if MAGICCAP_INTERNAL_TEMP_ICON is set.
    if let Ok(val) = std::env::var("MAGICCAP_INTERNAL_TEMP_ICON") {
        if val == "1" {
            crate::temp_icon::icond();
            return;
        }
    }

    // Create the shared application box.
    let leaky_box = Box::leak(Box::new(SharedApplication {
        context: FakeSend {
            value: WebContext::default().unwrap(),
        },
        protocol_handler: Box::new(RwLock::new(None)),
        webview: RwLock::new(None),
        tray_icon: RwLock::new(None),
        menu_event: RwLock::new(None),
        hotkey_wrapper: HotkeyWrapper::new(),
    }));
    let ptr = leaky_box as *mut SharedApplication;
    unsafe {
        let _ = SHARED_APPLICATION.set(&mut *ptr);
    }

    // Set the MAGICCAP_INTERNAL_MEMORY_ADDRESS env var.
    std::env::set_var(
        "MAGICCAP_INTERNAL_MEMORY_ADDRESS",
        (ptr as usize).to_string(),
    );

    // Set the context handler for the webview context.
    app()
        .context
        .value
        .register_uri_scheme("magiccap-internal", |req| {
            let protocol_handler = app().protocol_handler.read().unwrap().clone();
            if let Some(hn) = protocol_handler {
                hn(req);
            }
        });

    // In a thread, launch the application_reload function. This is because it can cause problems
    // if it blocks the main thread.
    run_thread(reload::application_reload);

    // Call gtk::main.
    gtk::main();
}

pub fn application_hydrate() {
    unsafe {
        // Check if SHARED_APPLICATION is set. If it is, application_init has already been called.
        if SHARED_APPLICATION.get().is_some() {
            // We are hydrated. Return now.
            return;
        }
    }

    // Get the MAGICCAP_INTERNAL_MEMORY_ADDRESS env var.
    let mem_addr = std::env::var("MAGICCAP_INTERNAL_MEMORY_ADDRESS")
        .unwrap()
        .parse::<usize>()
        .unwrap();

    // Turn it into a pointer.
    unsafe {
        let _ = SHARED_APPLICATION.set((mem_addr as *mut SharedApplication).as_mut().unwrap());
    }

    // In a thread, launch the application_reload function. This is because it can cause problems
    // if it blocks the main thread.
    run_thread(reload::application_reload);
}
