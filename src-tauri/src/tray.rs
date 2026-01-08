use crate::models::AppLifecycle;
use std::sync::atomic::Ordering;
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    webview::WebviewWindowBuilder,
    App, Emitter, LogicalSize, Manager, Runtime, Size, WebviewUrl,
};

/// Helper function to create or get the main window
fn get_or_create_window<R: Runtime>(
    app: &tauri::AppHandle<R>,
    width: f64,
    height: f64,
) -> Option<tauri::WebviewWindow<R>> {
    // Try to get existing window first
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_size(Size::Logical(LogicalSize { width, height }));
        let _ = window.show();
        let _ = window.set_focus();
        return Some(window);
    }

    // Window doesn't exist, create a new one
    match WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .title("ActioWatch")
        .inner_size(width, height)
        .decorations(false)
        .transparent(true)
        .build()
    {
        Ok(window) => {
            let _ = window.show();
            let _ = window.set_focus();
            Some(window)
        }
        Err(e) => {
            eprintln!("Failed to create window: {}", e);
            None
        }
    }
}

/// Destroy the main window to free WebView memory
fn destroy_window<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.destroy();
    }
}

pub fn create_tray<R: Runtime>(app: &mut App<R>) -> Result<tauri::tray::TrayIcon<R>, tauri::Error> {
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let hide_i = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
    let widget_i = MenuItem::with_id(app, "widget", "Widget Mode", true, None::<&str>)?;
    let standard_i = MenuItem::with_id(app, "standard", "Standard Mode", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_i, &hide_i, &standard_i, &widget_i, &quit_i])?;

    // Build tray icon with explicit icon for Windows
    let tray_builder = TrayIconBuilder::with_id("tray")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => {
                let app_state = app.state::<AppLifecycle>();
                app_state.is_quitting.store(true, Ordering::Relaxed);
                app.exit(0);
            }
            "show" => {
                // Recreate window if it was destroyed, or show if it exists
                let app_state = app.state::<AppLifecycle>();
                *app_state.current_view.lock().unwrap() = "process".to_string();

                if let Some(window) = get_or_create_window(app, 800.0, 600.0) {
                    // Also emit for existing windows
                    let _ = window.emit("view-change", "process");
                }
            }
            "hide" => {
                // Destroy window to free WebView memory (backend keeps running)
                destroy_window(app);
            }
            "widget" => {
                // Create/show window in widget mode
                let app_state = app.state::<AppLifecycle>();
                *app_state.current_view.lock().unwrap() = "widget".to_string();

                if let Some(window) = get_or_create_window(app, 380.0, 540.0) {
                    let _ = window.emit("view-change", "widget");
                }
            }
            "standard" => {
                // Create/show window in standard mode
                let app_state = app.state::<AppLifecycle>();
                *app_state.current_view.lock().unwrap() = "process".to_string();

                if let Some(window) = get_or_create_window(app, 800.0, 600.0) {
                    let _ = window.emit("view-change", "process");
                }
            }
            _ => {}
        });

    // Set icon explicitly for all platforms
    let tray_builder = tray_builder.icon(
        app.default_window_icon()
            .expect("Failed to load window icon")
            .clone(),
    );

    tray_builder.build(app)
}
