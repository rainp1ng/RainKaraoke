use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    pub tags: Option<String>,
    pub difficulty: Option<i32>,

    pub play_count: i32,
    pub last_played_at: Option<String>,

    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSong {
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub video_path: Option<String>,
    pub vocal_audio_path: Option<String>,
    pub instrumental_audio_path: Option<String>,
    pub lyrics_path: Option<String>,
    pub genre: Option<String>,
    pub language: Option<String>,
    pub tags: Option<Vec<String>>,
    pub duration: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSong {
    pub id: i64,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub video_path: Option<String>,
    pub vocal_audio_path: Option<String>,
    pub instrumental_audio_path: Option<String>,
    pub lyrics_path: Option<String>,
    pub genre: Option<String>,
    pub language: Option<String>,
    pub tags: Option<Vec<String>>,
}
