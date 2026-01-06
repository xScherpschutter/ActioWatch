use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    App, Emitter, LogicalSize, Manager, Runtime, Size,
};

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
                app.exit(0);
            }
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            "widget" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_size(Size::Logical(LogicalSize {
                        width: 320.0,
                        height: 480.0,
                    }));
                    let _ = window.emit("view-change", "widget");
                    let _ = window.show();
                }
            }
            "standard" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_size(Size::Logical(LogicalSize {
                        width: 800.0,
                        height: 600.0,
                    }));
                    let _ = window.emit("view-change", "process");
                    let _ = window.show();
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
