use tauri::State;
use crate::db::Database;
use crate::modules::AppState;
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
    pub volume: f64,
}

/// 歌曲信息（用于播放）
struct SongInfo {
    title: String,
    video_path: Option<String>,
    vocal_audio_path: Option<String>,
    instrumental_audio_path: Option<String>,
    duration: Option<i64>,
}

#[tauri::command]
pub fn play_song(
    songId: i64,
    startTime: Option<f64>,
    db: State<Database>,
    state: State<AppState>,
) -> Result<bool, String> {
    println!("[播放] 收到播放请求, song_id={}", songId);

    // 从数据库获取歌曲信息
    let conn = crate::db::get_connection(&db);

    let song_info = conn
        .query_row(
            "SELECT title, video_path, vocal_audio_path, instrumental_audio_path, duration FROM songs WHERE id = ?",
            [songId],
            |row| {
                Ok(SongInfo {
                    title: row.get(0)?,
                    video_path: row.get(1)?,
                    vocal_audio_path: row.get(2)?,
                    instrumental_audio_path: row.get(3)?,
                    duration: row.get(4)?,
                })
            },
        )
        .map_err(|e| format!("歌曲不存在: {}", e))?;

    println!("[播放] 歌曲: {}", song_info.title);
    println!("[播放] video_path: {:?}", song_info.video_path);
    println!("[播放] vocal_audio_path: {:?}", song_info.vocal_audio_path);
    println!("[播放] instrumental_audio_path: {:?}", song_info.instrumental_audio_path);

    // 获取媒体引擎并播放
    let mut media_engine = state.media_engine.lock().unwrap();

    // 根据原唱/伴唱模式选择音频路径
    let audio_path = if media_engine.get_state().is_vocal {
        // 原唱模式：优先使用原唱，没有原唱则回退到伴奏
        song_info.vocal_audio_path.clone()
            .or_else(|| {
                println!("[播放] 没有原唱音频，回退到伴奏");
                song_info.instrumental_audio_path.clone()
            })
    } else {
        // 伴唱模式：优先使用伴奏，没有伴奏则回退到原唱
        song_info.instrumental_audio_path.clone()
            .or_else(|| {
                println!("[播放] 没有伴奏音频，回退到原唱");
                song_info.vocal_audio_path.clone()
            })
    };

    println!("[播放] 选择的音频路径: {:?}", audio_path);

    // 如果没有独立音频，尝试播放视频文件中的音频
    let audio_to_play = audio_path.clone().or_else(|| {
        if let Some(ref video_path) = song_info.video_path {
            // 检查视频文件格式
            let path_lower = video_path.to_lowercase();
            if path_lower.ends_with(".mp4") || path_lower.ends_with(".mkv") || path_lower.ends_with(".avi") {
                println!("[播放] 视频文件 {} 需要视频播放器支持", video_path);
                // 对于视频文件，我们暂时返回视频路径，让 Rodio 尝试播放
                // 如果 Rodio 不支持，会返回错误
                Some(video_path.clone())
            } else {
                println!("[播放] 尝试播放视频文件中的音频: {}", video_path);
                Some(video_path.clone())
            }
        } else {
            None
        }
    });

    // 播放歌曲
    media_engine
        .play(songId, song_info.video_path.clone(), audio_to_play)
        .map_err(|e| format!("播放失败: {}", e))?;

    // 如果指定了开始时间，跳转到该位置
    if let Some(time) = startTime {
        if time > 0.0 {
            media_engine.seek(time).map_err(|e| format!("跳转失败: {}", e))?;
        }
    }

    // 更新播放次数
    let _ = conn.execute(
        "UPDATE songs SET play_count = play_count + 1, last_played_at = CURRENT_TIMESTAMP WHERE id = ?",
        [songId],
    );

    println!("[播放] 播放命令已发送");
    Ok(true)
}

#[tauri::command]
pub fn pause_song(state: State<AppState>) -> Result<bool, String> {
    let mut media_engine = state.media_engine.lock().unwrap();
    media_engine.pause().map_err(|e| format!("暂停失败: {}", e))?;
    Ok(true)
}

#[tauri::command]
pub fn resume_song(state: State<AppState>) -> Result<bool, String> {
    let mut media_engine = state.media_engine.lock().unwrap();
    media_engine.resume().map_err(|e| format!("继续播放失败: {}", e))?;
    Ok(true)
}

#[tauri::command]
pub fn stop_song(state: State<AppState>) -> Result<bool, String> {
    let mut media_engine = state.media_engine.lock().unwrap();
    media_engine.stop().map_err(|e| format!("停止失败: {}", e))?;
    Ok(true)
}

#[tauri::command]
pub fn seek_to(time: f64, state: State<AppState>) -> Result<bool, String> {
    let mut media_engine = state.media_engine.lock().unwrap();
    media_engine.seek(time).map_err(|e| format!("跳转失败: {}", e))?;
    Ok(true)
}

#[tauri::command]
pub fn toggle_vocal(enabled: bool, state: State<AppState>) -> Result<bool, String> {
    let mut media_engine = state.media_engine.lock().unwrap();
    media_engine.toggle_vocal(enabled).map_err(|e| format!("切换失败: {}", e))?;
    Ok(true)
}

#[tauri::command]
pub fn set_pitch(semitones: i32, state: State<AppState>) -> Result<bool, String> {
    let mut media_engine = state.media_engine.lock().unwrap();
    media_engine.set_pitch(semitones).map_err(|e| format!("设置音调失败: {}", e))?;
    Ok(true)
}

#[tauri::command]
pub fn set_speed(speed: f64, state: State<AppState>) -> Result<bool, String> {
    let mut media_engine = state.media_engine.lock().unwrap();
    media_engine.set_speed(speed).map_err(|e| format!("设置速度失败: {}", e))?;
    Ok(true)
}

#[tauri::command]
pub fn set_volume(volume: f64, state: State<AppState>) -> Result<bool, String> {
    let mut media_engine = state.media_engine.lock().unwrap();
    media_engine.set_volume(volume).map_err(|e| format!("设置音量失败: {}", e))?;
    Ok(true)
}

#[tauri::command]
pub fn get_playback_state(state: State<AppState>) -> Result<PlaybackState, String> {
    let media_engine = state.media_engine.lock().unwrap();
    let engine_state = media_engine.get_state();

    Ok(PlaybackState {
        status: engine_state.status.to_string(),
        current_song_id: engine_state.current_song_id,
        current_time: engine_state.current_time,
        duration: engine_state.duration,
        is_vocal: engine_state.is_vocal,
        pitch: engine_state.pitch,
        speed: engine_state.speed,
        volume: engine_state.volume,
    })
}

#[tauri::command]
pub fn update_playback_time(time: f64, state: State<AppState>) -> Result<(), String> {
    let mut media_engine = state.media_engine.lock().unwrap();
    media_engine.update_time(time);
    Ok(())
}
