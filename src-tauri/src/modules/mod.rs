use tauri::AppHandle;

pub mod media_engine;
pub mod audio_router;
pub mod vad;
pub mod lyrics_parser;
pub mod midi_handler;
pub mod interlude;

pub fn init(_app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // 初始化各个模块
    // media_engine::init(app_handle)?;

    Ok(())
}
