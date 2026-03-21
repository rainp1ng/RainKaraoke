use tauri::{AppHandle, Manager};
use std::sync::Mutex;

pub mod media_engine;
pub mod audio_router;
pub mod audio_processor;
pub mod vad;
pub mod lyrics_parser;
pub mod midi_handler;
pub mod interlude;
pub mod atmosphere;
pub mod effects;

use media_engine::MediaEngine;
use interlude::InterludeManager;
use atmosphere::AtmosphereManager;

/// 全局应用状态
pub struct AppState {
    pub media_engine: Mutex<MediaEngine>,
    pub interlude_manager: Mutex<InterludeManager>,
    pub atmosphere_manager: Mutex<AtmosphereManager>,
}

pub fn init(app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // 创建并注册媒体引擎
    let mut media_engine = MediaEngine::new();
    media_engine.set_app_handle(app_handle.clone());

    // 创建过场音乐管理器
    let mut interlude_manager = InterludeManager::new();
    interlude_manager.set_app_handle(app_handle.clone());

    // 创建气氛组管理器
    let mut atmosphere_manager = AtmosphereManager::new();
    atmosphere_manager.set_app_handle(app_handle.clone());

    app_handle.manage(AppState {
        media_engine: Mutex::new(media_engine),
        interlude_manager: Mutex::new(interlude_manager),
        atmosphere_manager: Mutex::new(atmosphere_manager),
    });

    Ok(())
}
