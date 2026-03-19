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

            // 播放控制命令
            commands::playback::play_song,
            commands::playback::pause_song,
            commands::playback::resume_song,
            commands::playback::stop_song,
            commands::playback::seek_to,
            commands::playback::toggle_vocal,
            commands::playback::set_pitch,
            commands::playback::set_speed,
            commands::playback::get_playback_state,
            commands::playback::update_playback_time,

            // 队列命令
            commands::queue::get_queue,
            commands::queue::add_to_queue,
            commands::queue::remove_from_queue,
            commands::queue::move_queue_item,
            commands::queue::clear_queue,
            commands::queue::play_next,

            // 过场音乐命令
            commands::interlude::get_interlude_tracks,
            commands::interlude::add_interlude_track,
            commands::interlude::delete_interlude_track,
            commands::interlude::set_interlude_volume,
            commands::interlude::get_interlude_state,

            // 气氛组命令
            commands::atmosphere::get_atmosphere_sounds,
            commands::atmosphere::add_atmosphere_sound,
            commands::atmosphere::update_atmosphere_sound,
            commands::atmosphere::delete_atmosphere_sound,
            commands::atmosphere::play_atmosphere_sound,
            commands::atmosphere::stop_atmosphere_sound,

            // MIDI命令
            commands::midi::get_midi_devices,
            commands::midi::connect_midi_device,
            commands::midi::disconnect_midi_device,
            commands::midi::get_midi_status,

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
            commands::effect::load_effect_preset,
            commands::effect::delete_effect_preset,
            commands::effect::bypass_all_effects,

            // 歌词命令
            commands::lyrics::get_lyrics,
            commands::lyrics::parse_lyrics_content,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
