import { create } from 'zustand'

export interface MidiBinding {
  note: number
  channel: number
}

export interface ShortcutConfig {
  playPause: string       // 播放/暂停
  nextSong: string        // 下一首
  prevSong: string        // 上一首
  stop: string            // 停止
  toggleVocal: string     // 切换原唱/伴奏
  fullscreen: string      // 全屏
  pip: string             // 画中画
}

export interface MidiConfig {
  playPause: MidiBinding | null
  nextSong: MidiBinding | null
  prevSong: MidiBinding | null
  stop: MidiBinding | null
  toggleVocal: MidiBinding | null
  fullscreen: MidiBinding | null
  pip: MidiBinding | null
}

interface ShortcutStore {
  config: ShortcutConfig
  midiConfig: MidiConfig
  learningKey: string | null  // 正在学习的键盘快捷键
  learningMidi: string | null  // 正在学习的 MIDI 快捷键
  setConfig: (config: Partial<ShortcutConfig>) => void
  setMidiConfig: (config: Partial<MidiConfig>) => void
  startLearning: (key: string) => void
  stopLearning: () => void
  startMidiLearning: (key: string) => void
  stopMidiLearning: () => void
  saveConfig: () => void
  loadConfig: () => void
}

const DEFAULT_CONFIG: ShortcutConfig = {
  playPause: 'Space',
  nextSong: 'KeyC',
  prevSong: 'KeyV',
  stop: 'KeyQ',
  toggleVocal: 'KeyX',
  fullscreen: 'KeyF',
  pip: 'KeyO',
}

const STORAGE_KEY = 'keyboardShortcuts'
const MIDI_STORAGE_KEY = 'midiShortcuts'

export const useShortcutStore = create<ShortcutStore>((set, get) => ({
  config: DEFAULT_CONFIG,
  midiConfig: {
    playPause: null,
    nextSong: null,
    prevSong: null,
    stop: null,
    toggleVocal: null,
    fullscreen: null,
    pip: null,
  },
  learningKey: null,
  learningMidi: null,

  setConfig: (newConfig) => {
    set((state) => ({
      config: { ...state.config, ...newConfig },
    }))
  },

  setMidiConfig: (newConfig) => {
    set((state) => ({
      midiConfig: { ...state.midiConfig, ...newConfig },
    }))
  },

  startLearning: (key) => {
    set({ learningKey: key, learningMidi: null })
  },

  stopLearning: () => {
    set({ learningKey: null })
  },

  startMidiLearning: (key) => {
    set({ learningMidi: key, learningKey: null })
  },

  stopMidiLearning: () => {
    set({ learningMidi: null })
  },

  saveConfig: () => {
    const { config, midiConfig } = get()
    localStorage.setItem(STORAGE_KEY, JSON.stringify(config))
    localStorage.setItem(MIDI_STORAGE_KEY, JSON.stringify(midiConfig))
  },

  loadConfig: () => {
    try {
      const stored = localStorage.getItem(STORAGE_KEY)
      if (stored) {
        const parsed = JSON.parse(stored)
        set({ config: { ...DEFAULT_CONFIG, ...parsed } })
      }
    } catch (e) {
      console.error('Failed to load shortcut config:', e)
    }

    try {
      const midiStored = localStorage.getItem(MIDI_STORAGE_KEY)
      if (midiStored) {
        const parsed = JSON.parse(midiStored)
        set((state) => ({
          midiConfig: { ...state.midiConfig, ...parsed }
        }))
      }
    } catch (e) {
      console.error('Failed to load midi config:', e)
    }
  },
}))
