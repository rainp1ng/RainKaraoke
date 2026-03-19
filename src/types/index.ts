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

export interface AtmosphereSound {
  id: number
  name: string
  filePath: string
  duration: number | null
  volume: number
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

  midiDeviceId: string | null
  midiEnabled: boolean
}

export interface PlaybackState {
  status: 'idle' | 'playing' | 'paused'
  currentSongId: number | null
  currentTime: number
  duration: number
  isVocal: boolean
  pitch: number
  speed: number
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
  time: number      // 毫秒
  duration?: number // 毫秒
  text: string
  words?: LyricsWord[]
}

export interface LyricsWord {
  time: number      // 毫秒
  duration: number  // 毫秒
  text: string
}

export interface Lyrics {
  format: 'lrc' | 'ksc' | 'txt'
  lines: LyricsLine[]
}

// ============ 效果器链类型 ============

export type EffectType =
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
  effectType: EffectType
  isEnabled: boolean
  parameters: Record<string, any>
}

export interface EffectChainConfig {
  inputDeviceId: string | null
  inputVolume: number
  monitorDeviceId: string | null
  streamDeviceId: string | null
  monitorVolume: number
  streamVolume: number
  bypassAll: boolean
}

export interface EffectPreset {
  id: number
  name: string
  description: string | null
  isDefault: boolean
}

// 效果器参数类型
export interface ReverbParams {
  roomSize: number      // 0-100
  damping: number       // 0-100
  wetLevel: number      // 0-100
  dryLevel: number      // 0-100
  preDelay: number      // 0-100 ms
}

export interface ChorusParams {
  rate: number          // 0.1-10 Hz
  depth: number         // 0-100 %
  mix: number           // 0-100 %
  voices: number        // 1-8
  spread: number        // 0-100 %
}

export interface EQBand {
  gain: number          // -12 to +12 dB
  frequency: number     // Hz
  q: number             // 0.1-10
}

export interface EQParams {
  low: EQBand
  lowMid: EQBand
  highMid: EQBand
  high: EQBand
}

export interface CompressorParams {
  threshold: number     // -60 to 0 dB
  ratio: number         // 1-20
  attack: number        // 0.1-100 ms
  release: number       // 10-1000 ms
  makeupGain: number    // 0-24 dB
}

export interface DelayParams {
  time: number          // 1-1000 ms
  feedback: number      // 0-90 %
  mix: number           // 0-100 %
  pingPong: boolean
}

export interface DeEsserParams {
  frequency: number     // 2000-12000 Hz
  threshold: number     // -40 to 0 dB
  range: number         // 0-24 dB
}

export interface ExciterParams {
  frequency: number     // 2000-16000 Hz
  harmonics: number     // 0-100 %
  mix: number           // 0-100 %
}

export interface NoiseGateParams {
  threshold: number     // -80 to -20 dB
  attack: number        // 0.1-50 ms
  release: number       // 10-500 ms
  range: number         // 0-80 dB
}

// 效果器类型信息
export const EFFECT_TYPES: { type: EffectType; name: string; icon: string }[] = [
  { type: 'reverb', name: '混响', icon: 'Waves' },
  { type: 'chorus', name: '合唱', icon: 'Users' },
  { type: 'eq', name: '均衡器', icon: 'Sliders' },
  { type: 'compressor', name: '压缩器', icon: 'ArrowDownWideNarrow' },
  { type: 'delay', name: '延迟', icon: 'Timer' },
  { type: 'deesser', name: '去齿音', icon: 'VolumeX' },
  { type: 'exciter', name: '激励器', icon: 'Sparkles' },
  { type: 'gate', name: '噪声门', icon: 'DoorClosed' },
]
