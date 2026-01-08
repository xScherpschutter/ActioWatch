use crate::models::AppLifecycle;
use tauri::State;

#[tauri::command]
pub fn get_current_view(state: State<AppLifecycle>) -> String {
    let view = state.current_view.lock().unwrap();
    view.clone()
}
