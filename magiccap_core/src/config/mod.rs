use base64::Engine;
use include_dir::{include_dir, Dir};
use uriparse::URI;
#[cfg(target_os = "macos")]
use cacao::{
    foundation::{NSString, nil},
    appkit::window::{Window, WindowConfig},
    webview::{WebView, WebViewConfig, WebViewDelegate},
};
#[cfg(target_os = "linux")]
use webkit2gtk::WebView;
use crate::database;
#[cfg(target_os = "macos")]
use crate::macos_delegate::app;
use crate::statics::run_thread;

// The folder which contains the frontend distribution.
static FRONTEND_DIST: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../frontend/dist");

// Defines sub-modules which are used to handle the config API.
mod api;
mod fs_proxy;
pub mod captures_html;

// Handles the frontend virtual host.
fn frontend_get(path: String) -> Option<Vec<u8>> {
    FRONTEND_DIST.
        get_file(path.trim_start_matches("/")).
        map(|f| f.contents().to_vec())
}

// Defines the function to handle message payloads.
fn message_sent(cpy: String) {
    // Take '{id}\n{type}\n' from the start, and then the rest is the body.
    let mut parts = cpy.splitn(3, "\n");
    let id = match parts.next() {
        Some(v) => v,
        None => return,
    }.to_string();
    let action_type = match parts.next() {
        Some(v) => v,
        None => return,
    };

    // Route based on the action type.
    let data = match action_type {
        "api" => {
            // Parse the body as JSON.
            let raw_body = match parts.next() {
                Some(v) => v,
                None => return,
            };

            // Parse the JSON.
            let obj: serde_json::Value = match serde_json::from_str(raw_body) {
                Ok(v) => v,
                Err(_) => return,
            };

            // Route the API call.
            api::handle_api_call(obj)
        }
        "fs_proxy" => {
            // Call the proxy handler.
            let fp = match parts.next() {
                Some(v) => v,
                None => return,
            };
            let mut ob: Vec<u8> = vec![0; 1];
            match fs_proxy::proxy_fp(fp) {
                Ok(mut v) => {
                    ob.append(&mut v);
                    ob[0] = 1;
                    ob
                },
                Err(err) => {
                    println!("[config.fs_proxy] Error proxying file path: {}", err);
                    ob
                },
            }
        },
        "captures_html" => captures_html::captures_html(),
        _ => return,
    };

    // Encode the data as base64.
    let b64 = base64::engine::general_purpose::STANDARD.encode(data);

    // Send the response on macOS.
    #[cfg(target_os = "macos")]
    {
        match app().delegate.webview.write().unwrap().as_ref() {
            Some(webview) => {
                // Yes, this is SUPER brutal.
                let webview = &webview.delegate.as_ref().unwrap().content;
                webview.objc.with_mut(|obj| unsafe {
                    let nsstr = NSString::new(
                        &format!("window.bridgeResponse({}, '{}');", id, b64)
                    );
                    let _: () = msg_send![obj, evaluateJavaScript: nsstr completionHandler: nil];
                });
            },
            None => {},
        };
    }

    // Send the response on Linux.
    #[cfg(target_os = "linux")]
    {
        use webkit2gtk::WebViewExt;

        crate::mainthread::main_thread_async(move || {
            match crate::linux_shared::app().webview.read().unwrap().as_ref() {
                Some(webview) => {
                    // Yes, this is SUPER brutal.
                    webview.value.run_javascript(&format!(
                        "window.bridgeResponse({}, '{}');", id, b64
                    ), None::<&gio::Cancellable>, |_| {});
                },
                None => {},
            }
        })
    }
}

// The delegate for the webview on macOS.
#[cfg(target_os = "macos")]
pub struct MagicCapConfigDelegate;

#[cfg(target_os = "macos")]
impl WebViewDelegate for MagicCapConfigDelegate {
    // Handle custom messages on macOS.
    fn on_message(&self, name: &str, body: &str) {
        use objc::{msg_send, sel, sel_impl};

        // Only supported item is bridge.
        if name != "bridge" {
            return;
        }

        // Copy the payload since we are doing this in a new thread.
        let cpy = body.to_string();

        // Spawn a new thread to handle the message.
        run_thread(move || message_sent(cpy));
    }

    // Handles the custom protocol requests on macOS.
    fn on_custom_protocol_request(&self, uri: &str) -> Option<Vec<u8>> {
        // Parse the URI.
        let uri = match URI::try_from(uri) {
            Ok(x) => x,
            Err(_) => return None,
        };

        // Check if the scheme matches.
        if uri.scheme() != "magiccap-internal" {
            return None;
        }

        // Route based on the host.
        match uri.host() {
            Some(v) => {
                match v.to_string().as_str() {
                    "frontend-dist" => frontend_get(uri.path().to_string()),
                    _ => return None,
                }
            }
            None => return None,
        }
    }
}

// Handles the window setup on macOS.
// !! WARNING !!: This is assumed to be on the main thread. If it is not, it will cause a crash.
#[cfg(target_os = "macos")]
pub fn setup_window(window: &Window) {
    // Set the window title.
    window.set_title("MagicCap");

    // Set the window size.
    window.set_minimum_content_size(1000., 600.);
    window.set_content_size(1000., 600.);
}

// Handles loading the config on macOS.
// !! WARNING !!: This is assumed to be on the main thread. If it is not, it will cause a crash.
#[cfg(target_os = "macos")]
pub fn open_config() {
    use crate::macos_delegate::ConfigWindow;

    // Check if the webview is already open.
    let webview_r = app().delegate.webview.read().unwrap();
    if webview_r.is_some() {
        // Focus the webview and return.
        webview_r.as_ref().unwrap().make_key_and_order_front();
        return;
    }
    drop(webview_r);

    // Get a write lock on the webview.
    let mut webview_w = app().delegate.webview.write().unwrap();
    if webview_w.is_some() {
        // This is a duplicate of the above to deal with the VERY rare case that a webview was opened
        // between the read unlock and the write lock.
        webview_w.as_ref().unwrap().make_key_and_order_front();
        return;
    }

    // Setup the webview config.
    let mut wv_config = WebViewConfig::default();
    wv_config.add_handler("bridge");
    wv_config.add_custom_protocol("magiccap-internal");

    // Allow devtools on debug builds.
    if cfg!(debug_assertions) {
        wv_config.enable_developer_extras();
    }

    // Create the webview.
    let webview = WebView::with (
        wv_config,
        MagicCapConfigDelegate,
    );

    // Load the webview.
    let html_url = match std::env::var("MAGICCAP_DEV_FRONTEND_URL") {
        Ok(v) => v,
        Err(_) => "magiccap-internal://frontend-dist/index.html".to_owned(),
    };
    webview.load_url(&html_url);

    // Create the window.
    let window: Window<ConfigWindow> = Window::with(
        WindowConfig::default(),
        ConfigWindow { content: webview },
    );

    // Set the webview in the app delegate.
    *webview_w = Some(window);
}

// Handles updating the webview if present.
#[cfg(target_os = "macos")]
pub fn update_webview_with_capture(capture_id: i64) {
    use objc::{msg_send, sel, sel_impl};

    let capture = match database::get_capture(capture_id) {
        Some(capture) => capture,
        None => return,
    };
    let html = crate::config::captures_html::generate_dom_node(capture).to_string();

    // Base64 the HTML.
    let html_base64 = base64::engine::general_purpose::STANDARD.encode(&html);

    match app().delegate.webview.write().unwrap().as_ref() {
        Some(webview) => {
            // Yes, this is SUPER brutal.
            let webview = &webview.delegate.as_ref().unwrap().content;
            webview.objc.with_mut(|obj| unsafe {
                let nsstr = NSString::new(
                    &format!(
                        "window.bridgeResponse(-1, '{}');", // see persistentHandlers in frontend/src/bridge/implementation.ts
                        html_base64)
                );
                let _: () = msg_send![obj, evaluateJavaScript: nsstr completionHandler: nil];
            });
        },
        None => {},
    }
}

// Handles creating the webview on Linux.
#[cfg(target_os = "linux")]
fn create_webview() -> WebView {
    use crate::linux_shared::app;
    use gtk::{prelude::*, Window};
    use webkit2gtk::{
        SettingsExt, URISchemeRequestExt, UserContentManager,
        UserContentManagerExt, WebViewExt, WebViewExtManual,
    };

    // Setup the JS bridge so that the webview can call out.
    let user_content_manager = &UserContentManager::new();
    user_content_manager.register_script_message_handler("bridge");
    user_content_manager.connect_script_message_received(Some("bridge"), |&_, resp| {
        let s = resp.js_value().unwrap().to_string();
        run_thread(move || message_sent(s));
    });

    // Initialize everything needed to handle the webview.
    let window = Window::new(gtk::WindowType::Toplevel);
    let wv = WebView::new_with_context_and_user_content_manager(
        &app().context.value, &user_content_manager,
    );
    let settings = WebViewExt::settings(&wv).unwrap();

    // Setup the custom protocol and load it.
    app().protocol_handler.write().unwrap().replace(&|req| {
        // Parse the URI.
        let uri = req.uri().unwrap();
        let uri = match URI::try_from(uri.as_str()) {
            Ok(x) => x,
            Err(_) => panic!("uri sent to magiccap-internal is not a valid URI"),
        };

        // Route based on the host.
        let value = match uri.host() {
            Some(v) => {
                match v.to_string().as_str() {
                    "frontend-dist" => {
                        frontend_get(uri.path().to_string())
                    },
                    _ => None,
                }
            }
            None => None,
        };

        // Handle returning the result.
        match value {
            Some(v) => {
                // Finish the request with the data.
                let input_stream = gio::MemoryInputStream::from_bytes(&glib::Bytes::from(&v));
                req.finish(&input_stream, v.len() as i64, None::<&str>);
            },
            None => {
                // Finish the request with an error.
                req.finish_error(&mut glib::Error::new(glib::FileError::Acces, "Resource not found"));
            }
        }
    });
    wv.load_uri("magiccap-internal://frontend-dist/index.html");

    // If this is a debug build, enable the developer extras.
    if cfg!(debug_assertions) {
        settings.set_enable_developer_extras(true);
    }

    // Mount it inside the window.
    window.add(&wv);

    // Handle window decorations.
    window.set_title("MagicCap");
    window.set_default_size(1000, 600);

    // When the window is closed, drop ourselves from the global app.
    window.connect_delete_event(|_, _| {
        // Drop the webview.
        let mut webview = app().webview.write().unwrap();
        webview.take();

        // Continue with the default behavior.
        glib::Propagation::Proceed
    });

    // Show the window.
    window.show_all();

    // Return the webview.
    wv
}

// Focuses the webview on Linux.
#[cfg(target_os = "linux")]
fn focus_webview(webview: &WebView) {
    use gtk::{current_event_time, prelude::WidgetExt};

    // Get the window this is relating to.
    let window = webview.toplevel().unwrap().window().unwrap();

    // Focus the window.
    window.focus(current_event_time());
}

// Handles loading the config on Linux.
// !! WARNING !!: This is assumed to be on the main thread. If it is not, it will cause a crash.
#[cfg(target_os = "linux")]
pub fn open_config() {
    use crate::linux_shared::{app, FakeSend};

    // Check if the webview is already open.
    let webview_r = app().webview.read().unwrap();
    if webview_r.is_some() {
        // Focus the webview and return.
        focus_webview(&webview_r.as_ref().unwrap().value);
        return;
    }
    drop(webview_r);

    // Get a write lock on the webview.
    let mut webview_w = app().webview.write().unwrap();
    if webview_w.is_some() {
        // This is a duplicate of the above to deal with the VERY rare case that a webview was opened
        // between the read unlock and the write lock.
        focus_webview(&webview_w.as_ref().unwrap().value);
        return;
    }

    // Create the webview.
    webview_w.replace(FakeSend { value: create_webview() });
}

// Handles updating the webview if present.
#[cfg(target_os = "linux")]
pub fn update_webview_with_capture(capture_id: i64) {
    use crate::{linux_shared::app, mainthread::main_thread_async};
    use webkit2gtk::WebViewExt;

    let capture = match database::get_capture(capture_id) {
        Some(capture) => capture,
        None => return,
    };
    let html = crate::config::captures_html::generate_dom_node(capture).to_string();
    let html_base64 = base64::engine::general_purpose::STANDARD.encode(&html);

    // Since we need the main thread on Linux, we push a async main thread task here.
    main_thread_async(move || {
        let read_ref = app().webview.read().unwrap();
        if let Some(webview) = read_ref.as_ref() {
            // Yes, this is SUPER brutal. We can actually do better on Linux, but we have to support macOS.
            webview.value.run_javascript(&format!(
                "window.bridgeResponse(-1, '{}');", // see persistentHandlers in frontend/src/bridge/implementation.ts
                html_base64
            ), None::<&gio::Cancellable>, |_| {});
        }
    });
}
