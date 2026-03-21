use serde::Serialize;
use tauri::{State, AppHandle, Manager};
use crate::db::Database;

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
pub fn connect_midi_device(app: AppHandle, db: State<Database>, device_name: String) -> Result<bool, String> {
    let mut handler = get_midi_handler().lock().unwrap();

    // 设置 app_handle 以便发送 MIDI 事件到前端
    handler.set_app_handle(app);

    handler.connect(&device_name)?;

    // 保存设备名称到数据库
    let conn = crate::db::get_connection(&db);
    conn.execute(
        "UPDATE audio_config SET midi_device_name = ?, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
        [&device_name],
    )
    .map_err(|e| format!("保存 MIDI 设备名称失败: {}", e))?;

    println!("[MIDI] 已连接并保存设备: {}", device_name);
    Ok(true)
}

#[tauri::command]
pub fn disconnect_midi_device(db: State<Database>) -> Result<bool, String> {
    let mut handler = get_midi_handler().lock().unwrap();
    handler.disconnect();

    // 清除数据库中的设备名称
    let conn = crate::db::get_connection(&db);
    conn.execute(
        "UPDATE audio_config SET midi_device_name = NULL, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
        [],
    )
    .map_err(|e| format!("清除 MIDI 设备名称失败: {}", e))?;

    println!("[MIDI] 已断开连接");
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

/// 获取已保存的 MIDI 设备名称
#[tauri::command]
pub fn get_saved_midi_device(db: State<Database>) -> Result<Option<String>, String> {
    let conn = crate::db::get_connection(&db);

    let device_name: Option<String> = conn
        .query_row(
            "SELECT midi_device_name FROM audio_config WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(None);

    Ok(device_name)
}

/// 尝试自动连接已保存的 MIDI 设备
#[tauri::command]
pub fn auto_connect_midi(app: AppHandle, db: State<Database>) -> Result<bool, String> {
    // 获取已保存的设备名称
    let conn = crate::db::get_connection(&db);
    let saved_device: Option<String> = conn
        .query_row(
            "SELECT midi_device_name FROM audio_config WHERE id = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(None);

    if let Some(device_name) = saved_device {
        println!("[MIDI] 尝试自动连接设备: {}", device_name);

        let mut handler = get_midi_handler().lock().unwrap();

        // 设置 app_handle 以便发送 MIDI 事件到前端
        handler.set_app_handle(app);

        // 先列出可用设备，检查目标设备是否存在
        let devices = handler.list_devices()?;
        let device_exists = devices.iter().any(|d| d.name == device_name);

        if device_exists {
            match handler.connect(&device_name) {
                Ok(_) => {
                    println!("[MIDI] 自动连接成功: {}", device_name);
                    return Ok(true);
                }
                Err(e) => {
                    println!("[MIDI] 自动连接失败: {}", e);
                }
            }
        } else {
            println!("[MIDI] 设备不存在: {}", device_name);
        }
    }

    Ok(false)
}
