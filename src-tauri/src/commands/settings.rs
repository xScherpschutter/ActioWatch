use crate::models::AppLifecycle;
use std::sync::atomic::Ordering;
use tauri::{State};

#[tauri::command]
pub fn get_notifications_enabled(state: State<AppLifecycle>) -> bool {
    state.notifications_enabled.load(Ordering::Relaxed)
}

#[tauri::command]
pub fn set_notifications_enabled(state: State<AppLifecycle>, enabled: bool) {
    state
        .notifications_enabled
        .store(enabled, Ordering::Relaxed);
}
