# RainKaraoke 数据库设计

## 数据库概述

- **数据库类型**：SQLite
- **ORM**：rusqlite (Rust)
- **位置**：`~/.rainkaraoke/data/rainkaraoke.db`

## 表结构

### 1. 歌曲表 (songs)

存储歌曲的基本信息和文件路径。

```sql
CREATE TABLE songs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- 基本信息
    title TEXT NOT NULL,              -- 歌曲名
    artist TEXT,                      -- 歌手
    album TEXT,                       -- 专辑
    duration INTEGER,                 -- 时长(秒)

    -- 文件路径
    video_path TEXT,                  -- 视频文件路径
    vocal_audio_path TEXT,            -- 原唱音频路径
    instrumental_audio_path TEXT,     -- 伴唱音频路径
    lyrics_path TEXT,                 -- 歌词文件路径

    -- 元数据
    lyrics_format TEXT,               -- 歌词格式: lrc, ksc, txt
    has_vocal BOOLEAN DEFAULT 0,      -- 是否有独立原唱
    has_instrumental BOOLEAN DEFAULT 0, -- 是否有独立伴唱

    -- 分类标签
    genre TEXT,                       -- 风格 (流行、摇滚、民谣等)
    language TEXT,                    -- 语言 (华语、欧美、日语、韩语等)
    tags TEXT,                        -- 自定义标签 (JSON数组)
    difficulty INTEGER,               -- 难度等级 1-5

    -- 统计信息
    play_count INTEGER DEFAULT 0,     -- 播放次数
    last_played_at DATETIME,          -- 最后播放时间

    -- 系统字段
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX idx_songs_title ON songs(title);
CREATE INDEX idx_songs_artist ON songs(artist);
CREATE INDEX idx_songs_genre ON songs(genre);
CREATE INDEX idx_songs_language ON songs(language);
```

### 2. 标签表 (tags)

用于灵活的标签系统。

```sql
CREATE TABLE tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,        -- 标签名称
    category TEXT,                    -- 分类: artist, genre, language, custom
    color TEXT,                       -- 显示颜色 (HEX格式)

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX idx_tags_name ON tags(name);
CREATE INDEX idx_tags_category ON tags(category);
```

### 3. 歌曲-标签关联表 (song_tags)

多对多关系。

```sql
CREATE TABLE song_tags (
    song_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,

    PRIMARY KEY (song_id, tag_id),
    FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);
```

### 4. 过场音乐表 (interlude_tracks)

```sql
CREATE TABLE interlude_tracks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT,                       -- 曲目名称
    file_path TEXT NOT NULL,          -- 文件路径
    duration INTEGER,                 -- 时长(秒)
    volume REAL DEFAULT 0.5,          -- 默认音量 0-1
    is_active BOOLEAN DEFAULT 1,      -- 是否启用
    play_count INTEGER DEFAULT 0,     -- 播放次数

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

### 5. 气氛组音频表 (atmosphere_sounds)

```sql
CREATE TABLE atmosphere_sounds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,               -- 音效名称
    file_path TEXT NOT NULL,          -- 文件路径
    duration INTEGER,                 -- 时长(秒)
    volume REAL DEFAULT 0.8,          -- 默认音量 0-1

    -- MIDI映射
    midi_note INTEGER,                -- MIDI音符编号 (0-127)
    midi_channel INTEGER DEFAULT 0,   -- MIDI通道 (0-15)

    -- 播放模式
    is_one_shot BOOLEAN DEFAULT 1,    -- true: 一次性播放, false: 循环播放
    color TEXT,                       -- UI显示颜色 (HEX格式)

    -- 排序
    sort_order INTEGER DEFAULT 0,     -- 排序顺序

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX idx_atmosphere_midi ON atmosphere_sounds(midi_note, midi_channel);
```

### 6. 播放队列表 (play_queue)

可选持久化，重启后恢复队列。

```sql
CREATE TABLE play_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    song_id INTEGER NOT NULL,
    position INTEGER NOT NULL,        -- 队列位置
    added_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX idx_queue_position ON play_queue(position);
```

### 7. 音频配置表 (audio_config)

存储全局音频设置。

```sql
CREATE TABLE audio_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- 只允许一行

    -- 输出设备
    default_output_device TEXT,       -- 默认输出设备ID
    interlude_output_device TEXT,     -- 过场音乐输出设备
    atmosphere_output_device TEXT,    -- 气氛组输出设备

    -- 音量设置
    master_volume REAL DEFAULT 0.8,   -- 主音量 0-1
    interlude_volume REAL DEFAULT 0.3, -- 过场音乐音量
    atmosphere_volume REAL DEFAULT 0.8, -- 气氛组音量

    -- Ducking设置
    ducking_enabled BOOLEAN DEFAULT 1,    -- 是否启用ducking
    ducking_threshold REAL DEFAULT 0.1,   -- VAD触发阈值
    ducking_ratio REAL DEFAULT 0.2,       -- 降低到的音量比例
    ducking_attack_ms INTEGER DEFAULT 100,  -- 淡出时间(ms)
    ducking_release_ms INTEGER DEFAULT 300, -- 淡入时间(ms)

    -- MIDI设置
    midi_device_id TEXT,              -- MIDI设备ID
    midi_enabled BOOLEAN DEFAULT 1,   -- 是否启用MIDI

    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 初始化默认配置
INSERT INTO audio_config (id) VALUES (1);
```

### 8. 播放历史表 (play_history)

记录播放历史，用于统计和推荐。

```sql
CREATE TABLE play_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    song_id INTEGER NOT NULL,
    played_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    duration_played INTEGER,          -- 实际播放时长(秒)

    FOREIGN KEY (song_id) REFERENCES songs(id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX idx_history_played_at ON play_history(played_at);
```

### 9. 效果器预设表 (effect_presets)

存储用户自定义的效果器预设。

```sql
CREATE TABLE effect_presets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,               -- 预设名称
    description TEXT,                 -- 预设描述
    is_default BOOLEAN DEFAULT 0,     -- 是否默认预设

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX idx_presets_name ON effect_presets(name);
```

### 10. 效果器链配置表 (effect_chain_config)

存储当前效果器链的配置。

```sql
CREATE TABLE effect_chain_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- 只允许一行

    -- 输入源配置
    input_device_id TEXT,             -- 输入设备ID
    input_volume REAL DEFAULT 1.0,    -- 输入增益

    -- 输出源配置
    monitor_device_id TEXT,           -- 监听输出设备
    stream_device_id TEXT,            -- 直播输出设备 (虚拟音频)
    monitor_volume REAL DEFAULT 0.8,  -- 监听音量
    stream_volume REAL DEFAULT 1.0,   -- 直播输出音量

    -- 总开关
    bypass_all BOOLEAN DEFAULT 0,     -- 是否旁通所有效果器

    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 初始化默认配置
INSERT INTO effect_chain_config (id) VALUES (1);
```

### 11. 效果器槽位表 (effect_slots)

存储效果器链中各个槽位的配置。

```sql
CREATE TABLE effect_slots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    slot_index INTEGER NOT NULL,      -- 槽位索引 (0-7)
    effect_type TEXT NOT NULL,        -- 效果器类型: reverb, chorus, eq, compressor, delay, deesser, exciter, gate
    is_enabled BOOLEAN DEFAULT 1,     -- 是否启用

    -- 参数存储 (JSON格式)
    parameters TEXT,                  -- 效果器参数

    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    UNIQUE(slot_index)
);

-- 索引
CREATE INDEX idx_slots_index ON effect_slots(slot_index);
```

### 12. 效果器参数示例

各效果器参数存储格式 (JSON):

```json
// 混响 (Reverb)
{
  "roomSize": 50,
  "damping": 30,
  "wetLevel": 30,
  "dryLevel": 70,
  "preDelay": 10
}

// 合唱 (Chorus)
{
  "rate": 1.5,
  "depth": 50,
  "mix": 30,
  "voices": 4,
  "spread": 50
}

// 均衡器 (EQ)
{
  "low": { "gain": 0, "frequency": 100, "q": 0.7 },
  "lowMid": { "gain": 0, "frequency": 500, "q": 0.7 },
  "highMid": { "gain": 0, "frequency": 4000, "q": 0.7 },
  "high": { "gain": 0, "frequency": 12000, "q": 0.7 }
}

// 压缩器 (Compressor)
{
  "threshold": -24,
  "ratio": 4,
  "attack": 10,
  "release": 100,
  "makeupGain": 0
}

// 延迟 (Delay)
{
  "time": 250,
  "feedback": 30,
  "mix": 20,
  "pingPong": false
}

// 去齿音 (De-Esser)
{
  "frequency": 6000,
  "threshold": -20,
  "range": 6
}

// 激励器 (Exciter)
{
  "frequency": 8000,
  "harmonics": 30,
  "mix": 20
}

// 噪声门 (Noise Gate)
{
  "threshold": -50,
  "attack": 1,
  "release": 50,
  "range": 40
}
```

## 数据关系图

```
┌─────────────┐       ┌─────────────┐       ┌─────────────┐
│   songs     │       │  song_tags  │       │    tags     │
├─────────────┤       ├─────────────┤       ├─────────────┤
│ id          │◄──────│ song_id     │       │ id          │
│ title       │       │ tag_id      │──────►│ name        │
│ artist      │       └─────────────┘       │ category    │
│ ...         │                             │ color       │
└──────┬──────┘                             └─────────────┘
       │
       │
       ▼
┌─────────────┐       ┌─────────────────┐
│ play_queue  │       │  play_history   │
├─────────────┤       ├─────────────────┤
│ id          │       │ id              │
│ song_id     │       │ song_id         │
│ position    │       │ played_at       │
└─────────────┘       └─────────────────┘

┌─────────────────────┐       ┌─────────────────────┐
│  interlude_tracks   │       │  atmosphere_sounds  │
├─────────────────────┤       ├─────────────────────┤
│ id                  │       │ id                  │
│ title               │       │ name                │
│ file_path           │       │ file_path           │
│ volume              │       │ midi_note           │
└─────────────────────┘       └─────────────────────┘

┌─────────────────────┐
│   audio_config      │
├─────────────────────┤
│ id (单行)           │
│ output_devices...   │
│ volumes...          │
│ ducking_settings... │
└─────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                     效果器链相关表                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────┐     ┌─────────────────────┐       │
│  │  effect_presets     │     │   effect_slots      │       │
│  ├─────────────────────┤     ├─────────────────────┤       │
│  │ id                  │     │ id                  │       │
│  │ name                │     │ slot_index (0-7)    │       │
│  │ description         │     │ effect_type         │       │
│  │ is_default          │     │ is_enabled          │       │
│  └─────────────────────┘     │ parameters (JSON)   │       │
│                              └─────────────────────┘       │
│                                                             │
│  ┌─────────────────────────────────────────────────┐       │
│  │           effect_chain_config (单行)             │       │
│  ├─────────────────────────────────────────────────┤       │
│  │ input_device_id      -- 输入设备                 │       │
│  │ input_volume         -- 输入增益                 │       │
│  │ monitor_device_id    -- 监听输出                 │       │
│  │ stream_device_id     -- 直播输出                 │       │
│  │ monitor_volume       -- 监听音量                 │       │
│  │ stream_volume        -- 直播音量                 │       │
│  │ bypass_all           -- 全部旁通                 │       │
│  └─────────────────────────────────────────────────┘       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Rust 数据模型

```rust
// src-tauri/src/models/song.rs
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    pub id: i64,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<i32>,

    pub video_path: Option<String>,
    pub vocal_audio_path: Option<String>,
    pub instrumental_audio_path: Option<String>,
    pub lyrics_path: Option<String>,

    pub lyrics_format: Option<String>,
    pub has_vocal: bool,
    pub has_instrumental: bool,

    pub genre: Option<String>,
    pub language: Option<String>,
    pub tags: Option<String>,  // JSON array
    pub difficulty: Option<i32>,

    pub play_count: i32,
    pub last_played_at: Option<NaiveDateTime>,

    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSong {
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<i32>,

    pub video_path: Option<String>,
    pub vocal_audio_path: Option<String>,
    pub instrumental_audio_path: Option<String>,
    pub lyrics_path: Option<String>,

    pub lyrics_format: Option<String>,
    pub genre: Option<String>,
    pub language: Option<String>,
    pub tags: Option<Vec<String>>,
    pub difficulty: Option<i32>,
}

// src-tauri/src/models/atmosphere_sound.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtmosphereSound {
    pub id: i64,
    pub name: String,
    pub file_path: String,
    pub duration: Option<i32>,
    pub volume: f32,

    pub midi_note: Option<i32>,
    pub midi_channel: i32,

    pub is_one_shot: bool,
    pub color: Option<String>,
    pub sort_order: i32,
}

// src-tauri/src/models/audio_config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub default_output_device: Option<String>,
    pub interlude_output_device: Option<String>,
    pub atmosphere_output_device: Option<String>,

    pub master_volume: f32,
    pub interlude_volume: f32,
    pub atmosphere_volume: f32,

    pub ducking_enabled: bool,
    pub ducking_threshold: f32,
    pub ducking_ratio: f32,
    pub ducking_attack_ms: i32,
    pub ducking_release_ms: i32,

    pub midi_device_id: Option<String>,
    pub midi_enabled: bool,
}
```
