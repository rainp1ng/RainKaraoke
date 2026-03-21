use tauri::Manager;

mod commands;
mod db;
mod models;
mod modules;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 初始化数据库
            let app_handle = app.handle();
            db::init_database(&app_handle)?;

            // 初始化模块
            modules::init(&app_handle)?;

            // 初始化全局音频状态
            app.manage(commands::effect::AppAudioState::new());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 媒体库命令
            commands::library::get_songs,
            commands::library::get_songs_count,
            commands::library::get_song_by_id,
            commands::library::add_song,
            commands::library::update_song,
            commands::library::delete_song,
            commands::library::import_songs,
            commands::library::import_single_file,
            commands::library::get_tags,
            commands::library::add_tag,
            commands::library::get_artists,
            commands::library::get_genres,
            commands::library::get_languages,
            commands::library::import_vocal,
            commands::library::import_lyrics,
            commands::library::update_song_metadata,

            // 播放控制命令
            commands::playback::play_song,
            commands::playback::pause_song,
            commands::playback::resume_song,
            commands::playback::stop_song,
            commands::playback::seek_to,
            commands::playback::toggle_vocal,
            commands::playback::set_pitch,
            commands::playback::set_speed,
            commands::playback::set_volume,
            commands::playback::get_playback_state,
            commands::playback::update_playback_time,

            // 队列命令
            commands::queue::get_queue,
            commands::queue::add_to_queue,
            commands::queue::remove_from_queue,
            commands::queue::move_queue_item,
            commands::queue::move_to_top,
            commands::queue::move_to_next,
            commands::queue::clear_queue,
            commands::queue::play_next,

            // 过场音乐命令
            commands::interlude::get_interlude_tracks,
            commands::interlude::add_interlude_track,
            commands::interlude::delete_interlude_track,
            commands::interlude::set_interlude_volume,
            commands::interlude::get_interlude_state,
            commands::interlude::play_interlude,
            commands::interlude::pause_interlude,
            commands::interlude::resume_interlude,
            commands::interlude::stop_interlude,

            // 气氛组命令
            commands::atmosphere::get_atmosphere_sounds,
            commands::atmosphere::add_atmosphere_sound,
            commands::atmosphere::update_atmosphere_sound,
            commands::atmosphere::delete_atmosphere_sound,
            commands::atmosphere::play_atmosphere_sound,
            commands::atmosphere::stop_atmosphere_sound,
            commands::atmosphere::set_atmosphere_volume,

            // MIDI命令
            commands::midi::get_midi_devices,
            commands::midi::connect_midi_device,
            commands::midi::disconnect_midi_device,
            commands::midi::get_midi_status,
            commands::midi::get_saved_midi_device,
            commands::midi::auto_connect_midi,

            // 音频设置命令
            commands::audio::get_audio_devices,
            commands::audio::get_audio_config,
            commands::audio::save_audio_config,
            commands::audio::get_default_input_device,
            commands::audio::get_default_output_device,

            // 效果器链命令
            commands::effect::get_effect_chain_config,
            commands::effect::save_effect_chain_config,
            commands::effect::get_effect_slots,
            commands::effect::set_effect_slot,
            commands::effect::update_effect_parameters,
            commands::effect::toggle_effect,
            commands::effect::move_effect_slot,
            commands::effect::clear_effect_slot,
            commands::effect::get_effect_presets,
            commands::effect::save_effect_preset,
            commands::effect::delete_effect_preset,
            commands::effect::bypass_all_effects,
            commands::effect::set_effect_midi,
            commands::effect::clear_effect_midi,

            // 音频设备和实时音频路由命令
            commands::effect::list_audio_input_devices,
            commands::effect::list_audio_output_devices,
            commands::effect::start_live_audio,
            commands::effect::stop_live_audio,
            commands::effect::set_effect_bypass,
            commands::effect::get_output_level,
            commands::effect::get_level_meter_value,
            commands::effect::move_effect_up,
            commands::effect::move_effect_down,
            commands::effect::get_live_audio_state,
            commands::effect::set_vocal_volume,
            commands::effect::set_instrument_volume,
            commands::effect::set_effect_input,
            commands::effect::set_vocal_channel,
            commands::effect::set_instrument_channel,
            commands::effect::get_ducking_debug_state,

            // 录音命令
            commands::effect::start_recording,
            commands::effect::stop_recording,
            commands::effect::get_recording_state,

            // 歌词命令
            commands::lyrics::get_lyrics,
            commands::lyrics::parse_lyrics_content,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
