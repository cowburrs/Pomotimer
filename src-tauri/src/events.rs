use std::time::Duration;

use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone, specta::Type, tauri_specta::Event)]
/// You will receive this event when someone joins your pomodoro
pub struct JoinEvent;

#[derive(Serialize, Deserialize, Debug, Clone, specta::Type)]
pub struct Message {
    host: bool,
    time: u32,
}

/// You will receive this event when someone messages in the pomodoro (not host specific)
#[derive(Serialize, Deserialize, Debug, Clone, specta::Type, tauri_specta::Event)]
pub struct MessageEvent(Message);
