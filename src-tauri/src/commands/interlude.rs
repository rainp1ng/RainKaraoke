use tauri::{State, Manager};
use crate::db::Database;
use crate::models::{InterludeTrack, NewInterludeTrack};
use crate::modules::AppState;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InterludeState {
    pub is_playing: bool,
    pub current_track_id: Option<i64>,
    pub volume: f32,
    pub ducking_active: bool,
}

#[tauri::command]
pub fn get_interlude_tracks(db: State<Database>) -> Result<Vec<InterludeTrack>, String> {
    let conn = crate::db::get_connection(&db);

    let tracks = conn
        .prepare("SELECT id, title, file_path, duration, volume, is_active, play_count FROM interlude_tracks WHERE is_active = 1")
        .map_err(|e| e.to_string())?
        .query_map([], |row| {
            Ok(InterludeTrack {
                id: row.get(0)?,
                title: row.get(1)?,
                file_path: row.get(2)?,
                duration: row.get(3)?,
                volume: row.get(4)?,
                is_active: row.get::<_, i32>(5)? == 1,
                play_count: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(tracks)
}

#[tauri::command]
pub fn add_interlude_track(db: State<Database>, track: NewInterludeTrack) -> Result<i64, String> {
    let conn = crate::db::get_connection(&db);
    let volume = track.volume.unwrap_or(0.5);

    conn.execute(
        "INSERT INTO interlude_tracks (title, file_path, volume) VALUES (?, ?, ?)",
        rusqlite::params![track.title, track.file_path, volume],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn delete_interlude_track(db: State<Database>, id: i64) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute("DELETE FROM interlude_tracks WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub fn set_interlude_volume(db: State<Database>, state: State<AppState>, volume: f32) -> Result<bool, String> {
    let conn = crate::db::get_connection(&db);

    conn.execute(
        "UPDATE audio_config SET interlude_volume = ?, updated_at = CURRENT_TIMESTAMP WHERE id = 1",
        [volume],
    )
    .map_err(|e| e.to_string())?;

    // 实时更新播放音量
    let mut manager = state.interlude_manager.lock().unwrap();
    manager.set_volume(volume)?;

    Ok(true)
}

#[tauri::command]
pub fn get_interlude_state(state: State<AppState>) -> Result<InterludeState, String> {
    let manager = state.interlude_manager.lock().unwrap();
    let inner_state = manager.get_state();

    Ok(InterludeState {
        is_playing: inner_state.is_playing,
        current_track_id: inner_state.current_track_id,
        volume: inner_state.volume,
        ducking_active: inner_state.ducking_active,
    })
}

#[tauri::command]
pub fn play_interlude(db: State<Database>, state: State<AppState>) -> Result<bool, String> {
    // 从数据库获取过场音乐列表
    let conn = crate::db::get_connection(&db);
    let tracks: Vec<crate::modules::interlude::InterludeTrack> = conn
        .prepare("SELECT id, title, file_path, volume FROM interlude_tracks WHERE is_active = 1")
        .map_err(|e| e.to_string())?
        .query_map([], |row| {
            Ok(crate::modules::interlude::InterludeTrack {
                id: row.get(0)?,
                title: row.get(1)?,
                file_path: row.get(2)?,
                volume: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    drop(conn);

    let mut manager = state.interlude_manager.lock().unwrap();
    manager.set_tracks(tracks);
    manager.start_random()?;

    Ok(true)
}

#[tauri::command]
pub fn pause_interlude(state: State<AppState>) -> Result<bool, String> {
    let mut manager = state.interlude_manager.lock().unwrap();
    manager.pause()?;
    Ok(true)
}

#[tauri::command]
pub fn resume_interlude(state: State<AppState>) -> Result<bool, String> {
    let mut manager = state.interlude_manager.lock().unwrap();
    manager.resume()?;
    Ok(true)
}

#[tauri::command]
pub fn stop_interlude(state: State<AppState>) -> Result<bool, String> {
    println!("[Interlude] stop_interlude 命令被调用");
    let mut manager = state.interlude_manager.lock().unwrap();

    // 打印当前状态
    let current_state = manager.get_state();
    println!("[Interlude] 当前状态: is_playing={}, current_track_id={:?}", current_state.is_playing, current_state.current_track_id);

    manager.stop()?;
    println!("[Interlude] stop() 已执行");

    let new_state = manager.get_state();
    println!("[Interlude] 停止后状态: is_playing={}", new_state.is_playing);

    Ok(true)
}
