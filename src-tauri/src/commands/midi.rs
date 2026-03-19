use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MidiDevice {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MidiStatus {
    pub connected: bool,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
}

/// 全局 MIDI 处理器实例
static MIDI_HANDLER: std::sync::OnceLock<std::sync::Mutex<crate::modules::midi_handler::MidiHandler>> =
    std::sync::OnceLock::new();

fn get_midi_handler() -> &'static std::sync::Mutex<crate::modules::midi_handler::MidiHandler> {
    MIDI_HANDLER.get_or_init(|| {
        std::sync::Mutex::new(crate::modules::midi_handler::MidiHandler::new())
    })
}

#[tauri::command]
pub fn get_midi_devices() -> Result<Vec<MidiDevice>, String> {
    let mut handler = get_midi_handler().lock().unwrap();
    let devices = handler.list_devices()?;

    Ok(devices
        .into_iter()
        .map(|d| MidiDevice {
            id: d.id,
            name: d.name,
        })
        .collect())
}

#[tauri::command]
pub fn connect_midi_device(device_name: String) -> Result<bool, String> {
    let mut handler = get_midi_handler().lock().unwrap();
    handler.connect(&device_name)?;
    Ok(true)
}

#[tauri::command]
pub fn disconnect_midi_device() -> Result<bool, String> {
    let mut handler = get_midi_handler().lock().unwrap();
    handler.disconnect();
    Ok(true)
}

#[tauri::command]
pub fn get_midi_status() -> Result<MidiStatus, String> {
    let handler = get_midi_handler().lock().unwrap();
    let status = handler.get_status();

    Ok(MidiStatus {
        connected: status.connected,
        device_id: None,
        device_name: status.device_name,
    })
}
