import { create } from 'zustand'
import type { AtmosphereSound, MidiMessageType } from '@/types'
import { atmosphereApi, midiApi, interludeApi } from '@/lib/api'
import { listen } from '@tauri-apps/api/event'

interface AtmosphereStore {
  sounds: AtmosphereSound[]
  isLoading: boolean
  midiStatus: {
    connected: boolean
    deviceId: string | null
    deviceName: string | null
  }
  midiDevices: { id: string; name: string }[]
  wasInterludePlaying: boolean // 记住过场音乐是否在播放

  loadSounds: () => Promise<void>
  addSound: (sound: {
    name: string
    filePath: string
    volume?: number
    midiMessageType?: MidiMessageType
    midiNote?: number
    midiChannel?: number
    isOneShot?: boolean
    color?: string
  }) => Promise<boolean>
  updateSound: (sound: {
    id: number
    name?: string
    volume?: number
    midiMessageType?: MidiMessageType
    midiNote?: number
    midiChannel?: number
    isOneShot?: boolean
    color?: string
  }) => Promise<boolean>
  deleteSound: (id: number) => Promise<void>
  playSound: (id: number) => Promise<void>
  stopSound: (id?: number) => Promise<void>

  loadMidiDevices: () => Promise<void>
  connectMidi: (deviceName: string) => Promise<boolean>
  disconnectMidi: () => Promise<boolean>
  loadMidiStatus: () => Promise<void>
  setupSoundEndedListener: () => Promise<() => void>
  initMidi: () => Promise<void>
}

export const useAtmosphereStore = create<AtmosphereStore>((set, get) => ({
  sounds: [],
  isLoading: false,
  midiStatus: {
    connected: false,
    deviceId: null,
    deviceName: null,
  },
  midiDevices: [],
  wasInterludePlaying: false,

  loadSounds: async () => {
    set({ isLoading: true })
    try {
      const sounds = await atmosphereApi.getAtmosphereSounds()
      set({ sounds, isLoading: false })
    } catch (error) {
      console.error('Failed to load atmosphere sounds:', error)
      set({ isLoading: false })
    }
  },

  addSound: async (sound) => {
    try {
      await atmosphereApi.addAtmosphereSound(sound)
      await get().loadSounds()
      return true
    } catch (error) {
      console.error('Failed to add atmosphere sound:', error)
      return false
    }
  },

  updateSound: async (sound) => {
    try {
      await atmosphereApi.updateAtmosphereSound(sound)
      await get().loadSounds()
      return true
    } catch (error) {
      console.error('Failed to update atmosphere sound:', error)
      return false
    }
  },

  deleteSound: async (id: number) => {
    try {
      await atmosphereApi.deleteAtmosphereSound(id)
      set((state) => ({
        sounds: state.sounds.filter((s) => s.id !== id),
      }))
    } catch (error) {
      console.error('Failed to delete atmosphere sound:', error)
    }
  },

  playSound: async (id: number) => {
    try {
      // 记住过场音乐是否在播放
      const interludeState = await interludeApi.getInterludeState()
      set({ wasInterludePlaying: interludeState.isPlaying })

      // 先停止过场音乐
      if (interludeState.isPlaying) {
        await interludeApi.stopInterlude()
      }

      await atmosphereApi.playAtmosphereSound(id)
    } catch (error) {
      console.error('Failed to play atmosphere sound:', error)
    }
  },

  stopSound: async (id?: number) => {
    try {
      await atmosphereApi.stopAtmosphereSound(id)
    } catch (error) {
      console.error('Failed to stop atmosphere sound:', error)
    }
  },

  loadMidiDevices: async () => {
    try {
      const devices = await midiApi.getMidiDevices()
      set({ midiDevices: devices })
    } catch (error) {
      console.error('Failed to load MIDI devices:', error)
    }
  },

  connectMidi: async (deviceName: string) => {
    try {
      const result = await midiApi.connectMidiDevice(deviceName)
      if (result) {
        await get().loadMidiStatus()
      }
      return result
    } catch (error) {
      console.error('Failed to connect MIDI device:', error)
      return false
    }
  },

  disconnectMidi: async () => {
    try {
      const result = await midiApi.disconnectMidiDevice()
      if (result) {
        await get().loadMidiStatus()
      }
      return result
    } catch (error) {
      console.error('Failed to disconnect MIDI device:', error)
      return false
    }
  },

  loadMidiStatus: async () => {
    try {
      const status = await midiApi.getMidiStatus()
      set({ midiStatus: status })
    } catch (error) {
      console.error('Failed to load MIDI status:', error)
    }
  },

  setupSoundEndedListener: async () => {
    const unlisten = await listen<number>('atmosphere:sound-ended', async () => {
      // 音效播放完成，恢复过场音乐
      const { wasInterludePlaying } = get()
      if (wasInterludePlaying) {
        try {
          await interludeApi.playInterlude()
          console.log('[气氛组] 已恢复过场音乐')
        } catch (e) {
          console.log('[气氛组] 恢复过场音乐失败')
        }
        set({ wasInterludePlaying: false })
      }
    })
    return unlisten
  },

  initMidi: async () => {
    await get().loadMidiStatus()
    await get().loadMidiDevices()
  },
}))
