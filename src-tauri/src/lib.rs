// TODO: https://www.blazingly.fast/
use core::time;
use iroh::{endpoint::presets, Endpoint};
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

mod networking;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn create_timer(state: tauri::State<Timers>) -> usize {
    let mut timers = state.timers.lock().unwrap();
    timers.push(Instant::now());
    timers.len() - 1
}

#[tauri::command]
fn get_timer(state: tauri::State<Timers>, index: usize) -> u64 {
    let timers = state.timers.lock().unwrap();
    timers[index].elapsed().as_secs()
}

struct Timers {
    timers: Mutex<Vec<Instant>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Timers {
            timers: Mutex::new(vec![Instant::now()]),
        })
        .manage(networking::create_pc())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            create_timer,
            get_timer,
            networking::host,
            networking::join,
            networking::sendmessage,
            networking::receivemessage,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
