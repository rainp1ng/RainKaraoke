use tauri::{AppHandle, Manager};
use rusqlite::Connection;
use std::sync::Mutex;
use tauri::State;

pub struct Database(Mutex<Connection>);

pub fn init_database(app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // 获取应用数据目录
    let app_dir = app_handle.path().app_data_dir()?;
    std::fs::create_dir_all(&app_dir)?;

    let db_path = app_dir.join("rainkaraoke.db");
    let conn = Connection::open(&db_path)?;

    // 执行数据库迁移
    migrate(&conn)?;

    // 将数据库连接存入状态
    app_handle.manage(Database(Mutex::new(conn)));

    Ok(())
}

fn migrate(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        -- 歌曲表
        CREATE TABLE IF NOT EXISTS songs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            artist TEXT,
            album TEXT,
            duration INTEGER,

            video_path TEXT,
            vocal_audio_path TEXT,
            instrumental_audio_path TEXT,
            lyrics_path TEXT,

            lyrics_format TEXT,
            has_vocal BOOLEAN DEFAULT 0,
            has_instrumental BOOLEAN DEFAULT 0,

            genre TEXT,
            language TEXT,
            tags TEXT,
            difficulty INTEGER,

            play_count INTEGER DEFAULT 0,
            last_played_at DATETIME,

            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_songs_title ON songs(title);
        CREATE INDEX IF NOT EXISTS idx_songs_artist ON songs(artist);

        -- 标签表
        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            category TEXT,
            color TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- 歌曲-标签关联表
        CREATE TABLE IF NOT EXISTS song_tags (
            song_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL,
            PRIMARY KEY (song_id, tag_id),
            FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE,
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );

        -- 过场音乐表
        CREATE TABLE IF NOT EXISTS interlude_tracks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT,
            file_path TEXT NOT NULL,
            duration INTEGER,
            volume REAL DEFAULT 0.5,
            is_active BOOLEAN DEFAULT 1,
            play_count INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- 气氛组音频表
        CREATE TABLE IF NOT EXISTS atmosphere_sounds (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            duration INTEGER,
            volume REAL DEFAULT 0.8,
            midi_message_type TEXT DEFAULT 'NOTE',
            midi_note INTEGER,
            midi_channel INTEGER DEFAULT 0,
            is_one_shot BOOLEAN DEFAULT 1,
            color TEXT,
            sort_order INTEGER DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_atmosphere_midi ON atmosphere_sounds(midi_message_type, midi_note, midi_channel);

        -- 迁移：添加 midi_message_type 列（如果不存在）
        INSERT OR IGNORE INTO atmosphere_sounds (id, name, file_path, volume, midi_message_type)
        SELECT 0, 'migration_placeholder', '', 0, 'NOTE' WHERE 0;

        -- 实际迁移在后续版本通过 ALTER TABLE 添加

        -- 播放队列表
        CREATE TABLE IF NOT EXISTS play_queue (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            song_id INTEGER NOT NULL,
            position INTEGER NOT NULL,
            added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE
        );

        -- 音频配置表
        CREATE TABLE IF NOT EXISTS audio_config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            default_output_device TEXT,
            interlude_output_device TEXT,
            atmosphere_output_device TEXT,
            master_volume REAL DEFAULT 0.8,
            interlude_volume REAL DEFAULT 0.3,
            atmosphere_volume REAL DEFAULT 0.8,
            ducking_enabled BOOLEAN DEFAULT 1,
            ducking_threshold REAL DEFAULT 0.01,
            ducking_ratio REAL DEFAULT 0.1,
            ducking_attack_ms INTEGER DEFAULT 100,
            ducking_release_ms INTEGER DEFAULT 300,
            midi_device_id TEXT,
            midi_device_name TEXT,
            midi_enabled BOOLEAN DEFAULT 1,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- 播放历史表
        CREATE TABLE IF NOT EXISTS play_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            song_id INTEGER NOT NULL,
            played_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            duration_played INTEGER,
            FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_history_played_at ON play_history(played_at);

        -- 效果器预设表
        CREATE TABLE IF NOT EXISTS effect_presets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            description TEXT,
            is_default BOOLEAN DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- 效果器链配置表
        CREATE TABLE IF NOT EXISTS effect_chain_config (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            input_device_id TEXT,
            input_volume REAL DEFAULT 1.0,
            monitor_device_id TEXT,
            stream_device_id TEXT,
            monitor_volume REAL DEFAULT 0.8,
            stream_volume REAL DEFAULT 1.0,
            bypass_all BOOLEAN DEFAULT 0,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        -- 效果器槽位表
        CREATE TABLE IF NOT EXISTS effect_slots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            slot_index INTEGER NOT NULL UNIQUE,
            effect_type TEXT NOT NULL,
            is_enabled BOOLEAN DEFAULT 1,
            parameters TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE INDEX IF NOT EXISTS idx_effect_slots_index ON effect_slots(slot_index);

        -- 初始化默认配置
        INSERT OR IGNORE INTO audio_config (id) VALUES (1);
        INSERT OR IGNORE INTO effect_chain_config (id) VALUES (1);
        "#,
    )?;

    // 迁移：添加 midi_message_type 列（如果不存在）
    // 检查列是否存在
    let column_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('atmosphere_sounds') WHERE name='midi_message_type'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !column_exists {
        conn.execute(
            "ALTER TABLE atmosphere_sounds ADD COLUMN midi_message_type TEXT DEFAULT 'NOTE'",
            [],
        )?;
        println!("[数据库] 已添加 midi_message_type 列");
    }

    // 迁移：添加 midi_device_name 列（如果不存在）
    let midi_name_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('audio_config') WHERE name='midi_device_name'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !midi_name_exists {
        conn.execute(
            "ALTER TABLE audio_config ADD COLUMN midi_device_name TEXT",
            [],
        )?;
        println!("[数据库] 已添加 midi_device_name 列");
    }

    // 迁移：添加 effect_chain_config 新字段
    // vocal_input_device
    let vocal_input_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('effect_chain_config') WHERE name='vocal_input_device'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !vocal_input_exists {
        conn.execute(
            "ALTER TABLE effect_chain_config ADD COLUMN vocal_input_device TEXT",
            [],
        )?;
        println!("[数据库] 已添加 vocal_input_device 列");
    }

    // instrument_input_device
    let instrument_input_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('effect_chain_config') WHERE name='instrument_input_device'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !instrument_input_exists {
        conn.execute(
            "ALTER TABLE effect_chain_config ADD COLUMN instrument_input_device TEXT",
            [],
        )?;
        println!("[数据库] 已添加 instrument_input_device 列");
    }

    // vocal_volume
    let vocal_volume_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('effect_chain_config') WHERE name='vocal_volume'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !vocal_volume_exists {
        conn.execute(
            "ALTER TABLE effect_chain_config ADD COLUMN vocal_volume REAL DEFAULT 0.8",
            [],
        )?;
        println!("[数据库] 已添加 vocal_volume 列");
    }

    // instrument_volume
    let instrument_volume_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('effect_chain_config') WHERE name='instrument_volume'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !instrument_volume_exists {
        conn.execute(
            "ALTER TABLE effect_chain_config ADD COLUMN instrument_volume REAL DEFAULT 0.8",
            [],
        )?;
        println!("[数据库] 已添加 instrument_volume 列");
    }

    // effect_input
    let effect_input_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('effect_chain_config') WHERE name='effect_input'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !effect_input_exists {
        conn.execute(
            "ALTER TABLE effect_chain_config ADD COLUMN effect_input TEXT DEFAULT 'vocal'",
            [],
        )?;
        println!("[数据库] 已添加 effect_input 列");
    }

    // recording_path
    let recording_path_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('effect_chain_config') WHERE name='recording_path'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !recording_path_exists {
        conn.execute(
            "ALTER TABLE effect_chain_config ADD COLUMN recording_path TEXT",
            [],
        )?;
        println!("[数据库] 已添加 recording_path 列");
    }

    // 迁移：为 effect_slots 添加 MIDI 控制字段
    let midi_note_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('effect_slots') WHERE name='midi_note'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !midi_note_exists {
        conn.execute(
            "ALTER TABLE effect_slots ADD COLUMN midi_note INTEGER",
            [],
        )?;
        println!("[数据库] 已添加 effect_slots.midi_note 列");
    }

    let midi_channel_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('effect_slots') WHERE name='midi_channel'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !midi_channel_exists {
        conn.execute(
            "ALTER TABLE effect_slots ADD COLUMN midi_channel INTEGER DEFAULT 0",
            [],
        )?;
        println!("[数据库] 已添加 effect_slots.midi_channel 列");
    }

    // 迁移：为 audio_config 添加 ducking_recovery_delay 字段
    let ducking_recovery_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('audio_config') WHERE name='ducking_recovery_delay'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !ducking_recovery_exists {
        conn.execute(
            "ALTER TABLE audio_config ADD COLUMN ducking_recovery_delay INTEGER DEFAULT 8",
            [],
        )?;
        println!("[数据库] 已添加 audio_config.ducking_recovery_delay 列");
    }

    // 迁移：为 effect_chain_config 添加通道配置字段
    let vocal_channel_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('effect_chain_config') WHERE name='vocal_input_channel'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !vocal_channel_exists {
        conn.execute(
            "ALTER TABLE effect_chain_config ADD COLUMN vocal_input_channel INTEGER DEFAULT 0",
            [],
        )?;
        println!("[数据库] 已添加 effect_chain_config.vocal_input_channel 列");
    }

    let instrument_channel_exists: bool = conn
        .query_row(
            "SELECT COUNT(*) > 0 FROM pragma_table_info('effect_chain_config') WHERE name='instrument_input_channel'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !instrument_channel_exists {
        conn.execute(
            "ALTER TABLE effect_chain_config ADD COLUMN instrument_input_channel INTEGER DEFAULT 1",
            [],
        )?;
        println!("[数据库] 已添加 effect_chain_config.instrument_input_channel 列");
    }

    // 迁移：更新 ducking 默认值（如果当前值是旧的默认值）
    // 检查是否需要更新 ducking 默认值
    let need_update_ducking: bool = conn
        .query_row(
            "SELECT ducking_threshold >= 0.05 OR ducking_ratio > 0.5 OR ducking_recovery_delay = 8 FROM audio_config WHERE id = 1",
            [],
            |row| row.get::<_, i32>(0),
        )
        .unwrap_or(1) == 1;

    if need_update_ducking {
        conn.execute(
            "UPDATE audio_config SET \
             ducking_threshold = CASE WHEN ducking_threshold >= 0.05 THEN 0.01 ELSE ducking_threshold END, \
             ducking_ratio = CASE WHEN ducking_ratio > 0.5 THEN 0.1 ELSE ducking_ratio END, \
             ducking_recovery_delay = CASE WHEN ducking_recovery_delay = 8 THEN 3 ELSE ducking_recovery_delay END \
             WHERE id = 1",
            [],
        )?;
        println!("[数据库] 已更新 ducking 默认值: threshold=0.01, ratio=0.1, recovery_delay=3");
    }

    Ok(())
}

pub fn get_connection<'a>(db: &'a State<'a, Database>) -> std::sync::MutexGuard<'a, Connection> {
    db.0.lock().unwrap()
}
