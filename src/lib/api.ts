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
  LiveAudioConfig,
  LiveAudioState,
  RecordingResult,
  DeviceInfo,
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

  async importVocal(songId: number, filePath: string): Promise<boolean> {
    return invoke('import_vocal', { songId, filePath })
  },

  async importLyrics(songId: number, filePath: string): Promise<boolean> {
    return invoke('import_lyrics', { songId, filePath })
  },

  async updateSongMetadata(
    songId: number,
    title?: string,
    artist?: string,
    album?: string
  ): Promise<boolean> {
    return invoke('update_song_metadata', { songId, title, artist, album })
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

  async setVolume(volume: number): Promise<boolean> {
    return invoke('set_volume', { volume })
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

  async moveToTop(queueId: number): Promise<boolean> {
    return invoke('move_to_top', { queueId })
  },

  async moveToNext(queueId: number, currentSongId: number): Promise<boolean> {
    return invoke('move_to_next', { queueId, currentSongId })
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
        filePath: track.filePath,
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

  async playInterlude(): Promise<boolean> {
    return invoke('play_interlude')
  },

  async pauseInterlude(): Promise<boolean> {
    return invoke('pause_interlude')
  },

  async resumeInterlude(): Promise<boolean> {
    return invoke('resume_interlude')
  },

  async stopInterlude(): Promise<boolean> {
    return invoke('stop_interlude')
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
    midiMessageType?: string
    midiNote?: number
    midiChannel?: number
    isOneShot?: boolean
    color?: string
  }): Promise<number> {
    return invoke('add_atmosphere_sound', {
      sound: {
        name: sound.name,
        filePath: sound.filePath,
        volume: sound.volume,
        midiMessageType: sound.midiMessageType,
        midiNote: sound.midiNote,
        midiChannel: sound.midiChannel,
        isOneShot: sound.isOneShot,
        color: sound.color,
      }
    })
  },

  async updateAtmosphereSound(sound: {
    id: number
    name?: string
    volume?: number
    midiMessageType?: string
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
        midiMessageType: sound.midiMessageType,
        midiNote: sound.midiNote,
        midiChannel: sound.midiChannel,
        isOneShot: sound.isOneShot,
        color: sound.color,
        sortOrder: sound.sortOrder,
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

  async setAtmosphereVolume(volume: number): Promise<boolean> {
    return invoke('set_atmosphere_volume', { volume })
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

  async getSavedMidiDevice(): Promise<string | null> {
    return invoke('get_saved_midi_device')
  },

  async autoConnectMidi(): Promise<boolean> {
    return invoke('auto_connect_midi')
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
        defaultOutputDevice: config.defaultOutputDevice,
        interludeOutputDevice: config.interludeOutputDevice,
        atmosphereOutputDevice: config.atmosphereOutputDevice,
        masterVolume: config.masterVolume,
        interludeVolume: config.interludeVolume,
        atmosphereVolume: config.atmosphereVolume,
        duckingEnabled: config.duckingEnabled,
        duckingThreshold: config.duckingThreshold,
        duckingRatio: config.duckingRatio,
        duckingAttackMs: config.duckingAttackMs,
        duckingReleaseMs: config.duckingReleaseMs,
        duckingRecoveryDelay: config.duckingRecoveryDelay,
        midiDeviceId: config.midiDeviceId,
        midiEnabled: config.midiEnabled,
        atmosphereStopMidiMessageType: config.atmosphereStopMidiMessageType,
        atmosphereStopMidiNote: config.atmosphereStopMidiNote,
        atmosphereStopMidiChannel: config.atmosphereStopMidiChannel,
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
        inputDeviceId: config.inputDeviceId,
        inputVolume: config.inputVolume,
        monitorDeviceId: config.monitorDeviceId,
        streamDeviceId: config.streamDeviceId,
        monitorVolume: config.monitorVolume,
        streamVolume: config.streamVolume,
        bypassAll: config.bypassAll,
        vocalInputDevice: config.vocalInputDevice,
        instrumentInputDevice: config.instrumentInputDevice,
        vocalInputChannel: config.vocalInputChannel,
        instrumentInputChannel: config.instrumentInputChannel,
        vocalVolume: config.vocalVolume,
        instrumentVolume: config.instrumentVolume,
        effectInput: config.effectInput,
        recordingPath: config.recordingPath,
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
        slotIndex: slot.slotIndex,
        effectType: slot.effectType,
        enabled: slot.enabled,
        parameters: slot.parameters ? JSON.stringify(slot.parameters) : null,
      }
    })
  },

  async updateEffectParameters(slotIndex: number, parameters: Record<string, any>): Promise<boolean> {
    return invoke('update_effect_parameters', {
      params: {
        slotIndex,
        parameters: JSON.stringify(parameters)
      }
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

  async deleteEffectPreset(presetId: number): Promise<boolean> {
    return invoke('delete_effect_preset', { presetId })
  },

  async bypassAllEffects(bypass: boolean): Promise<boolean> {
    return invoke('bypass_all_effects', { bypass })
  },

  // 音频设备列表（新 API，包含通道数）
  async listAudioInputDevices(): Promise<DeviceInfo[]> {
    return invoke('list_audio_input_devices')
  },

  async listAudioOutputDevices(): Promise<DeviceInfo[]> {
    return invoke('list_audio_output_devices')
  },

  // 实时音频路由
  async startLiveAudio(config: LiveAudioConfig): Promise<boolean> {
    return invoke('start_live_audio', { config })
  },

  async stopLiveAudio(): Promise<boolean> {
    return invoke('stop_live_audio')
  },

  async setEffectBypass(bypass: boolean): Promise<boolean> {
    return invoke('set_effect_bypass', { bypass })
  },

  async getOutputLevel(): Promise<number> {
    return invoke('get_output_level')
  },

  async getLevelMeterValue(slotIndex: number): Promise<number | null> {
    return invoke('get_level_meter_value', { slotIndex })
  },

  async moveEffectUp(slotIndex: number): Promise<boolean> {
    return invoke('move_effect_up', { slotIndex })
  },

  async moveEffectDown(slotIndex: number): Promise<boolean> {
    return invoke('move_effect_down', { slotIndex })
  },

  async getLiveAudioState(): Promise<LiveAudioState> {
    return invoke('get_live_audio_state')
  },

  async setVocalVolume(volume: number): Promise<boolean> {
    return invoke('set_vocal_volume', { volume })
  },

  async setInstrumentVolume(volume: number): Promise<boolean> {
    return invoke('set_instrument_volume', { volume })
  },

  async setEffectInput(effectInput: 'vocal' | 'instrument' | 'none'): Promise<boolean> {
    return invoke('set_effect_input', { effectInput })
  },

  async setVocalChannel(channel: number): Promise<boolean> {
    return invoke('set_vocal_channel', { channel })
  },

  async setInstrumentChannel(channel: number): Promise<boolean> {
    return invoke('set_instrument_channel', { channel })
  },

  // 录音控制
  async startRecording(vocalPath?: string, instrumentPath?: string): Promise<boolean> {
    return invoke('start_recording', {
      config: {
        vocal_path: vocalPath,
        instrument_path: instrumentPath,
      }
    })
  },

  async stopRecording(): Promise<RecordingResult> {
    return invoke('stop_recording')
  },

  async getRecordingState(): Promise<boolean> {
    return invoke('get_recording_state')
  },

  // 效果器 MIDI 学习
  async setEffectMidi(slotIndex: number, midiNote: number, midiChannel: number): Promise<boolean> {
    return invoke('set_effect_midi', { slotIndex, midiNote, midiChannel })
  },

  async clearEffectMidi(slotIndex: number): Promise<boolean> {
    return invoke('clear_effect_midi', { slotIndex })
  },

  // Ducking 调试
  async getDuckingDebugState(): Promise<{
    enabled: boolean
    interludePlaying: boolean
    isDucking: boolean
    threshold: number
    ratio: number
    recoveryDelay: number
    releaseStart: number
    elapsedSinceReleaseStart: number
    remainingTime: number
  }> {
    return invoke('get_ducking_debug_state')
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
