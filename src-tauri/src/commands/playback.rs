use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackState {
    pub status: String,
    pub current_song_id: Option<i64>,
    pub current_time: f64,
    pub duration: f64,
    pub is_vocal: bool,
    pub pitch: i32,
    pub speed: f64,
}

#[tauri::command]
pub fn play_song(song_id: i64, start_time: Option<f64>) -> Result<bool, String> {
    let _ = (song_id, start_time);
    // TODO: 从数据库获取歌曲信息并播放
    Ok(true)
}

#[tauri::command]
pub fn pause_song() -> Result<bool, String> {
    // TODO: 实现暂停逻辑
    Ok(true)
}

#[tauri::command]
pub fn resume_song() -> Result<bool, String> {
    // TODO: 实现继续播放逻辑
    Ok(true)
}

#[tauri::command]
pub fn stop_song() -> Result<bool, String> {
    // TODO: 实现停止逻辑
    Ok(true)
}

#[tauri::command]
pub fn seek_to(time: f64) -> Result<bool, String> {
    let _ = time;
    // TODO: 实现跳转逻辑
    Ok(true)
}

#[tauri::command]
pub fn toggle_vocal(enabled: bool) -> Result<bool, String> {
    let _ = enabled;
    // TODO: 实现原唱/伴唱切换逻辑
    Ok(true)
}

#[tauri::command]
pub fn set_pitch(semitones: i32) -> Result<bool, String> {
    let _ = semitones;
    // TODO: 实现音调设置逻辑
    Ok(true)
}

#[tauri::command]
pub fn set_speed(speed: f64) -> Result<bool, String> {
    let _ = speed;
    // TODO: 实现速度设置逻辑
    Ok(true)
}

#[tauri::command]
pub fn get_playback_state() -> Result<PlaybackState, String> {
    // TODO: 实现获取播放状态逻辑
    Ok(PlaybackState {
        status: "idle".to_string(),
        current_song_id: None,
        current_time: 0.0,
        duration: 0.0,
        is_vocal: true,
        pitch: 0,
        speed: 1.0,
    })
}

#[tauri::command]
pub fn update_playback_time(time: f64) -> Result<(), String> {
    let _ = time;
    // TODO: 更新播放时间
    Ok(())
}
