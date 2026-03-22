export interface Song {
  id: number
  title: string
  artist: string | null
  album: string | null
  duration: number | null

  videoPath: string | null
  vocalAudioPath: string | null
  instrumentalAudioPath: string | null
  lyricsPath: string | null

  lyricsFormat: string | null
  hasVocal: boolean
  hasInstrumental: boolean

  genre: string | null
  language: string | null
  tags: string[]
  difficulty: number | null

  playCount: number
  lastPlayedAt: string | null

  createdAt: string
  updatedAt: string
}

export interface Tag {
  id: number
  name: string
  category: string | null
  color: string | null
}

export interface QueueItem {
  id: number
  songId: number
  position: number
  song?: Song
}

export interface InterludeTrack {
  id: number
  title: string | null
  filePath: string
  duration: number | null
  volume: number
  isActive: boolean
  playCount: number
}

export type MidiMessageType = 'NOTE' | 'CC' | 'PC'

export interface AtmosphereSound {
  id: number
  name: string
  filePath: string
  duration: number | null
  volume: number
  midiMessageType: MidiMessageType
  midiNote: number | null
  midiChannel: number
  isOneShot: boolean
  color: string | null
  sortOrder: number
}

export interface AudioConfig {
  defaultOutputDevice: string | null
  interludeOutputDevice: string | null
  atmosphereOutputDevice: string | null

  masterVolume: number
  interludeVolume: number
  atmosphereVolume: number

  duckingEnabled: boolean
  duckingThreshold: number
  duckingRatio: number
  duckingAttackMs: number
  duckingReleaseMs: number
  duckingRecoveryDelay: number // 恢复延迟（秒，1-9）

  midiDeviceId: string | null
  midiEnabled: boolean

  // 气氛组停止按钮 MIDI 配置
  atmosphereStopMidiMessageType: string | null
  atmosphereStopMidiNote: number | null
  atmosphereStopMidiChannel: number | null
}

export interface PlaybackState {
  status: 'idle' | 'playing' | 'paused'
  currentSongId: number | null
  currentTime: number
  duration: number
  isVocal: boolean
  pitch: number
  speed: number
  volume: number
}

export interface InterludeState {
  isPlaying: boolean
  currentTrackId: number | null
  volume: number
  duckingActive: boolean
}

export interface MidiStatus {
  connected: boolean
  deviceId: string | null
  deviceName: string | null
}

export interface AudioDevice {
  id: string
  name: string
  type: 'input' | 'output'
  isDefault: boolean
  channels: number
}

export interface LyricsLine {
  time: number
  duration?: number
  text: string
  words?: LyricsWord[]
}

export interface LyricsWord {
  time: number
  duration: number
  text: string
}

export interface Lyrics {
  format: 'lrc' | 'ksc' | 'txt'
  lines: LyricsLine[]
}

// ============ 效果器链类型 ============

export type EffectType =
  | 'gain'
  | 'reverb'
  | 'chorus'
  | 'eq'
  | 'compressor'
  | 'delay'
  | 'deesser'
  | 'exciter'
  | 'gate'

export interface EffectSlot {
  id: number
  slotIndex: number
  effectType: string
  isEnabled: boolean
  parameters: Record<string, any>
  /** MIDI 音符编号 (0-127)，用于通过 MIDI 控制开关 */
  midiNote: number | null
  /** MIDI 通道 (0-15) */
  midiChannel: number
}

export interface EffectChainConfig {
  inputDeviceId: string | null
  inputVolume: number
  monitorDeviceId: string | null
  streamDeviceId: string | null
  monitorVolume: number
  streamVolume: number
  bypassAll: boolean
  vocalInputDevice: string | null
  instrumentInputDevice: string | null
  vocalInputChannel: number
  instrumentInputChannel: number
  vocalVolume: number
  instrumentVolume: number
  effectInput: 'vocal' | 'instrument' | 'none'
  recordingPath: string | null
}

export interface EffectPreset {
  id: number
  name: string
  description: string | null
  isDefault: boolean
}

// 设备信息（包含通道数）
export interface DeviceInfo {
  name: string
  channels: number
  sampleRate: number
  isDefault: boolean
}

// 实时音频配置
export interface LiveAudioConfig {
  vocalInputDevice: string | null
  vocalInputChannel: number
  instrumentInputDevice: string | null
  instrumentInputChannel: number
  monitorOutputDevice: string
  streamOutputDevice: string | null
  vocalVolume: number
  instrumentVolume: number
  effectInput: 'vocal' | 'instrument' | 'none'
  monitorVolume: number
  streamVolume: number
}

// 实时音频状态
export interface LiveAudioState {
  isRunning: boolean
  config: LiveAudioConfig
  vocalRecording: boolean
  instrumentRecording: boolean
}

// 录音结果
export interface RecordingResult {
  vocalPath: string | null
  instrumentPath: string | null
}

// 效果器类型信息
export const EFFECT_TYPES: { type: string; name: string; icon: string }[] = [
  { type: 'gain', name: '增益', icon: 'Volume2' },
  { type: 'eq', name: '均衡器', icon: 'Sliders' },
  { type: 'compressor', name: '压缩器', icon: 'ArrowDownWideNarrow' },
  { type: 'deesser', name: '去齿音', icon: 'VolumeX' },
  { type: 'exciter', name: '激励器', icon: 'Sparkles' },
  { type: 'reverb', name: '混响', icon: 'Waves' },
  { type: 'chorus', name: '合唱', icon: 'Users' },
  { type: 'delay', name: '延迟', icon: 'Timer' },
  { type: 'gate', name: '噪声门', icon: 'DoorClosed' },
  { type: 'levelmeter', name: '电平表', icon: 'Activity' },
]
