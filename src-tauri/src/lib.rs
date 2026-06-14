// TODO: Clippy pedantic
// TODO: https://www.blazingly.fast/

mod networking;
mod timer;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let handlers = create_bindings().invoke_handler();

    tauri::Builder::default()
        .manage(timer::Timers::new())
        .manage(networking::create_pc())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(handlers)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn create_bindings() -> tauri_specta::Builder<tauri::Wry> {
    let builder = tauri_specta::Builder::<tauri::Wry>::new()
        .semantic_types(specta_typescript::semantic::Configuration::default())
        .commands(tauri_specta::collect_commands![
            timer::greet,
            timer::create_timer,
            timer::get_timer,
            networking::host,
            networking::join,
            networking::sendmessage,
            networking::receivemessage,
        ])
        // .typ::<Custom>()
        // .typ::<Testing>()
        .events(tauri_specta::collect_events![]);
    return builder;
}
#[test]
fn export_bindings() {
    use specta_typescript::{JSDoc, Layout};
    create_bindings()
        .export(
            JSDoc::default().layout(Layout::Files),
            "../src/bindings-js-files",
        )
        .expect("Failed to export typescript bindings");
    println!("Created bindings!")
}
