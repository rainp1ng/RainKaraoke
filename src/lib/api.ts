import { invoke } from '@tauri-apps/api/core'
import type {
  Song,
  Tag,
  QueueItem,
  InterludeTrack,
  AtmosphereSound,
  AudioConfig,
  EffectSlot,
  EffectChainConfig,
  EffectPreset,
  PlaybackState,
} from '@/types'

// ============ 媒体库 API ============

export const libraryApi = {
  async getSongs(params: {
    page?: number
    pageSize?: number
    search?: string
    artist?: string
    genre?: string
    language?: string
    sortBy?: string
    sortOrder?: string
  }): Promise<Song[]> {
    return invoke('get_songs', {
      page: params.page,
      pageSize: params.pageSize,
      search: params.search,
      artist: params.artist,
      genre: params.genre,
      language: params.language,
      sortBy: params.sortBy,
      sortOrder: params.sortOrder,
    })
  },

  async getSongsCount(params: {
    search?: string
    artist?: string
    genre?: string
    language?: string
  }): Promise<number> {
    return invoke('get_songs_count', params)
  },

  async getSongById(id: number): Promise<Song | null> {
    return invoke('get_song_by_id', { id })
  },

  async addSong(song: {
    title: string
    artist?: string
    album?: string
    videoPath?: string
    vocalAudioPath?: string
    instrumentalAudioPath?: string
    lyricsPath?: string
    genre?: string
    language?: string
    tags?: string[]
    duration?: number
  }): Promise<number> {
    return invoke('add_song', {
      song: {
        title: song.title,
        artist: song.artist,
        album: song.album,
        video_path: song.videoPath,
        vocal_audio_path: song.vocalAudioPath,
        instrumental_audio_path: song.instrumentalAudioPath,
        lyrics_path: song.lyricsPath,
        genre: song.genre,
        language: song.language,
        tags: song.tags,
        duration: song.duration,
      }
    })
  },

  async updateSong(song: {
    id: number
    title?: string
    artist?: string
    album?: string
    videoPath?: string
    vocalAudioPath?: string
    instrumentalAudioPath?: string
    lyricsPath?: string
    genre?: string
    language?: string
    tags?: string[]
  }): Promise<boolean> {
    return invoke('update_song', {
      song: {
        id: song.id,
        title: song.title,
        artist: song.artist,
        album: song.album,
        video_path: song.videoPath,
        vocal_audio_path: song.vocalAudioPath,
        instrumental_audio_path: song.instrumentalAudioPath,
        lyrics_path: song.lyricsPath,
        genre: song.genre,
        language: song.language,
        tags: song.tags,
      }
    })
  },

  async deleteSong(id: number): Promise<boolean> {
    return invoke('delete_song', { id })
  },

  async importSongs(directory: string, recursive: boolean = true): Promise<{
    success: number
    skipped: number
    failed: number
    errors: string[]
  }> {
    return invoke('import_songs', { directory, recursive })
  },

  async importSingleFile(filePath: string): Promise<number> {
    return invoke('import_single_file', { filePath })
  },

  async getTags(category?: string): Promise<Tag[]> {
    return invoke('get_tags', { category })
  },

  async addTag(name: string, category?: string, color?: string): Promise<number> {
    return invoke('add_tag', { name, category, color })
  },

  async getArtists(): Promise<string[]> {
    return invoke('get_artists')
  },

  async getGenres(): Promise<string[]> {
    return invoke('get_genres')
  },

  async getLanguages(): Promise<string[]> {
    return invoke('get_languages')
  },
}

// ============ 播放控制 API ============

export const playbackApi = {
  async playSong(songId: number, startTime?: number): Promise<boolean> {
    return invoke('play_song', { songId, startTime })
  },

  async pauseSong(): Promise<boolean> {
    return invoke('pause_song')
  },

  async resumeSong(): Promise<boolean> {
    return invoke('resume_song')
  },

  async stopSong(): Promise<boolean> {
    return invoke('stop_song')
  },

  async seekTo(time: number): Promise<boolean> {
    return invoke('seek_to', { time })
  },

  async toggleVocal(enabled: boolean): Promise<boolean> {
    return invoke('toggle_vocal', { enabled })
  },

  async setPitch(semitones: number): Promise<boolean> {
    return invoke('set_pitch', { semitones })
  },

  async setSpeed(speed: number): Promise<boolean> {
    return invoke('set_speed', { speed })
  },

  async getPlaybackState(): Promise<PlaybackState> {
    return invoke('get_playback_state')
  },
}

// ============ 队列 API ============

export const queueApi = {
  async getQueue(): Promise<QueueItem[]> {
    return invoke('get_queue')
  },

  async addToQueue(songId: number, position?: number): Promise<number> {
    return invoke('add_to_queue', { songId, position })
  },

  async removeFromQueue(queueId: number): Promise<boolean> {
    return invoke('remove_from_queue', { queueId })
  },

  async moveQueueItem(queueId: number, newPosition: number): Promise<boolean> {
    return invoke('move_queue_item', { queueId, newPosition })
  },

  async clearQueue(): Promise<boolean> {
    return invoke('clear_queue')
  },

  async playNext(): Promise<boolean> {
    return invoke('play_next')
  },
}

// ============ 过场音乐 API ============

export const interludeApi = {
  async getInterludeTracks(): Promise<InterludeTrack[]> {
    return invoke('get_interlude_tracks')
  },

  async addInterludeTrack(track: {
    title?: string
    filePath: string
    volume?: number
  }): Promise<number> {
    return invoke('add_interlude_track', {
      track: {
        title: track.title,
        file_path: track.filePath,
        volume: track.volume,
      }
    })
  },

  async deleteInterludeTrack(id: number): Promise<boolean> {
    return invoke('delete_interlude_track', { id })
  },

  async setInterludeVolume(volume: number): Promise<boolean> {
    return invoke('set_interlude_volume', { volume })
  },

  async getInterludeState(): Promise<{
    isPlaying: boolean
    currentTrackId: number | null
    volume: number
    duckingActive: boolean
  }> {
    return invoke('get_interlude_state')
  },
}

// ============ 气氛组 API ============

export const atmosphereApi = {
  async getAtmosphereSounds(): Promise<AtmosphereSound[]> {
    return invoke('get_atmosphere_sounds')
  },

  async addAtmosphereSound(sound: {
    name: string
    filePath: string
    volume?: number
    midiNote?: number
    midiChannel?: number
    isOneShot?: boolean
    color?: string
  }): Promise<number> {
    return invoke('add_atmosphere_sound', {
      sound: {
        name: sound.name,
        file_path: sound.filePath,
        volume: sound.volume,
        midi_note: sound.midiNote,
        midi_channel: sound.midiChannel,
        is_one_shot: sound.isOneShot,
        color: sound.color,
      }
    })
  },

  async updateAtmosphereSound(sound: {
    id: number
    name?: string
    volume?: number
    midiNote?: number
    midiChannel?: number
    isOneShot?: boolean
    color?: string
    sortOrder?: number
  }): Promise<boolean> {
    return invoke('update_atmosphere_sound', {
      sound: {
        id: sound.id,
        name: sound.name,
        volume: sound.volume,
        midi_note: sound.midiNote,
        midi_channel: sound.midiChannel,
        is_one_shot: sound.isOneShot,
        color: sound.color,
        sort_order: sound.sortOrder,
      }
    })
  },

  async deleteAtmosphereSound(id: number): Promise<boolean> {
    return invoke('delete_atmosphere_sound', { id })
  },

  async playAtmosphereSound(id: number): Promise<boolean> {
    return invoke('play_atmosphere_sound', { id })
  },

  async stopAtmosphereSound(id?: number): Promise<boolean> {
    return invoke('stop_atmosphere_sound', { id })
  },
}

// ============ MIDI API ============

export const midiApi = {
  async getMidiDevices(): Promise<{ id: string; name: string }[]> {
    return invoke('get_midi_devices')
  },

  async connectMidiDevice(deviceName: string): Promise<boolean> {
    return invoke('connect_midi_device', { deviceName })
  },

  async disconnectMidiDevice(): Promise<boolean> {
    return invoke('disconnect_midi_device')
  },

  async getMidiStatus(): Promise<{
    connected: boolean
    deviceId: string | null
    deviceName: string | null
  }> {
    return invoke('get_midi_status')
  },
}

// ============ 音频设置 API ============

export const audioApi = {
  async getAudioDevices(): Promise<{
    id: string
    name: string
    type: 'input' | 'output'
    isDefault: boolean
    channels: number
  }[]> {
    return invoke('get_audio_devices')
  },

  async getAudioConfig(): Promise<AudioConfig> {
    return invoke('get_audio_config')
  },

  async saveAudioConfig(config: Partial<AudioConfig>): Promise<boolean> {
    return invoke('save_audio_config', {
      config: {
        default_output_device: config.defaultOutputDevice,
        interlude_output_device: config.interludeOutputDevice,
        atmosphere_output_device: config.atmosphereOutputDevice,
        master_volume: config.masterVolume,
        interlude_volume: config.interludeVolume,
        atmosphere_volume: config.atmosphereVolume,
        ducking_enabled: config.duckingEnabled,
        ducking_threshold: config.duckingThreshold,
        ducking_ratio: config.duckingRatio,
        ducking_attack_ms: config.duckingAttackMs,
        ducking_release_ms: config.duckingReleaseMs,
        midi_device_id: config.midiDeviceId,
        midi_enabled: config.midiEnabled,
      }
    })
  },
}

// ============ 效果器链 API ============

export const effectApi = {
  async getEffectChainConfig(): Promise<EffectChainConfig> {
    return invoke('get_effect_chain_config')
  },

  async saveEffectChainConfig(config: Partial<EffectChainConfig>): Promise<boolean> {
    return invoke('save_effect_chain_config', {
      config: {
        input_device_id: config.inputDeviceId,
        input_volume: config.inputVolume,
        monitor_device_id: config.monitorDeviceId,
        stream_device_id: config.streamDeviceId,
        monitor_volume: config.monitorVolume,
        stream_volume: config.streamVolume,
        bypass_all: config.bypassAll,
      }
    })
  },

  async getEffectSlots(): Promise<EffectSlot[]> {
    return invoke('get_effect_slots')
  },

  async setEffectSlot(slot: {
    slotIndex: number
    effectType: string
    enabled?: boolean
    parameters?: Record<string, any>
  }): Promise<boolean> {
    return invoke('set_effect_slot', {
      slot: {
        slot_index: slot.slotIndex,
        effect_type: slot.effectType,
        enabled: slot.enabled,
        parameters: slot.parameters ? JSON.stringify(slot.parameters) : null,
      }
    })
  },

  async updateEffectParameters(slotIndex: number, parameters: Record<string, any>): Promise<boolean> {
    return invoke('update_effect_parameters', {
      slotIndex,
      parameters: JSON.stringify(parameters)
    })
  },

  async toggleEffect(slotIndex: number, enabled: boolean): Promise<boolean> {
    return invoke('toggle_effect', { slotIndex, enabled })
  },

  async moveEffectSlot(fromIndex: number, toIndex: number): Promise<boolean> {
    return invoke('move_effect_slot', { fromIndex, toIndex })
  },

  async clearEffectSlot(slotIndex: number): Promise<boolean> {
    return invoke('clear_effect_slot', { slotIndex })
  },

  async getEffectPresets(): Promise<EffectPreset[]> {
    return invoke('get_effect_presets')
  },

  async saveEffectPreset(name: string, description?: string): Promise<number> {
    return invoke('save_effect_preset', { name, description })
  },

  async loadEffectPreset(presetId: number): Promise<boolean> {
    return invoke('load_effect_preset', { presetId })
  },

  async deleteEffectPreset(presetId: number): Promise<boolean> {
    return invoke('delete_effect_preset', { presetId })
  },

  async bypassAllEffects(bypass: boolean): Promise<boolean> {
    return invoke('bypass_all_effects', { bypass })
  },
}

// ============ 歌词 API ============

export const lyricsApi = {
  async getLyrics(songId: number): Promise<{
    format: 'lrc' | 'ksc' | 'txt'
    lines: {
      time: number
      duration?: number
      text: string
      words?: { time: number; duration: number; text: string }[]
    }[]
  } | null> {
    return invoke('get_lyrics', { songId })
  },

  async parseLyricsContent(content: string, format: 'lrc' | 'ksc' | 'txt'): Promise<{
    format: string
    lines: {
      time: number
      duration?: number
      text: string
      words?: { time: number; duration: number; text: string }[]
    }[]
  }> {
    return invoke('parse_lyrics_content', { content, format })
  },
}
