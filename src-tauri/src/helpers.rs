#[tauri::command]
#[specta::specta]
/// print to stdout instead of the in terminal.
pub fn print(text: &str) {
    println!("{}", text);
}
