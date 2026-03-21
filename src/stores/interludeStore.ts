import { create } from 'zustand'
import type { InterludeTrack, InterludeState } from '@/types'
import { interludeApi } from '@/lib/api'

interface InterludeStore {
  tracks: InterludeTrack[]
  state: InterludeState
  isLoading: boolean

  loadTracks: () => Promise<void>
  addTrack: (filePath: string, title?: string) => Promise<boolean>
  deleteTrack: (id: number) => Promise<void>
  setVolume: (volume: number) => Promise<void>
  loadState: () => Promise<void>
  play: () => Promise<void>
  pause: () => Promise<void>
  stop: () => Promise<void>
}

export const useInterludeStore = create<InterludeStore>((set, get) => ({
  tracks: [],
  state: {
    isPlaying: false,
    currentTrackId: null,
    volume: 0.3,
    duckingActive: false,
  },
  isLoading: false,

  loadTracks: async () => {
    set({ isLoading: true })
    try {
      const tracks = await interludeApi.getInterludeTracks()
      set({ tracks, isLoading: false })
    } catch (error) {
      console.error('Failed to load interlude tracks:', error)
      set({ isLoading: false })
    }
  },

  addTrack: async (filePath: string, title?: string) => {
    try {
      await interludeApi.addInterludeTrack({
        title: title || filePath.split('/').pop()?.replace(/\.[^/.]+$/, '') || '未命名',
        filePath,
        volume: 0.5,
      })
      await get().loadTracks()
      return true
    } catch (error) {
      console.error('Failed to add interlude track:', error)
      return false
    }
  },

  deleteTrack: async (id: number) => {
    try {
      await interludeApi.deleteInterludeTrack(id)
      set((state) => ({
        tracks: state.tracks.filter((t) => t.id !== id),
      }))
    } catch (error) {
      console.error('Failed to delete interlude track:', error)
    }
  },

  setVolume: async (volume: number) => {
    try {
      await interludeApi.setInterludeVolume(volume)
      set((state) => ({
        state: { ...state.state, volume },
      }))
    } catch (error) {
      console.error('Failed to set interlude volume:', error)
    }
  },

  loadState: async () => {
    try {
      const state = await interludeApi.getInterludeState()
      set({ state })
    } catch (error) {
      console.error('Failed to load interlude state:', error)
    }
  },

  play: async () => {
    try {
      await interludeApi.playInterlude()
      set((state) => ({ state: { ...state.state, isPlaying: true } }))
    } catch (error) {
      console.error('Failed to play interlude:', error)
    }
  },

  pause: async () => {
    try {
      await interludeApi.pauseInterlude()
      set((state) => ({ state: { ...state.state, isPlaying: false } }))
    } catch (error) {
      console.error('Failed to pause interlude:', error)
    }
  },

  stop: async () => {
    try {
      await interludeApi.stopInterlude()
      set((state) => ({ state: { ...state.state, isPlaying: false, currentTrackId: null } }))
    } catch (error) {
      console.error('Failed to stop interlude:', error)
    }
  },
}))
