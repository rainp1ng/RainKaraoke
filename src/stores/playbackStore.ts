import { create } from 'zustand'
import type { Song, PlaybackState } from '@/types'
import { playbackApi } from '@/lib/api'

interface PlaybackStore extends PlaybackState {
  currentSong: Song | null

  play: (song: Song) => Promise<void>
  pause: () => Promise<void>
  resume: () => Promise<void>
  stop: () => Promise<void>
  seek: (time: number) => Promise<void>
  toggleVocal: (isVocal: boolean) => Promise<void>
  setPitch: (semitones: number) => Promise<void>
  setSpeed: (speed: number) => Promise<void>
  updateState: () => Promise<void>
  setCurrentTime: (time: number) => void
}

export const usePlaybackStore = create<PlaybackStore>((set, get) => ({
  status: 'idle',
  currentSongId: null,
  currentTime: 0,
  duration: 0,
  isVocal: true,
  pitch: 0,
  speed: 1.0,
  currentSong: null,

  play: async (song: Song) => {
    try {
      await playbackApi.playSong(song.id)
      set({
        status: 'playing',
        currentSongId: song.id,
        currentSong: song,
        duration: song.duration || 0,
        currentTime: 0,
      })
    } catch (error) {
      console.error('Failed to play:', error)
    }
  },

  pause: async () => {
    try {
      await playbackApi.pauseSong()
      set({ status: 'paused' })
    } catch (error) {
      console.error('Failed to pause:', error)
    }
  },

  resume: async () => {
    try {
      await playbackApi.resumeSong()
      set({ status: 'playing' })
    } catch (error) {
      console.error('Failed to resume:', error)
    }
  },

  stop: async () => {
    try {
      await playbackApi.stopSong()
      set({
        status: 'idle',
        currentSongId: null,
        currentSong: null,
        currentTime: 0,
      })
    } catch (error) {
      console.error('Failed to stop:', error)
    }
  },

  seek: async (time: number) => {
    try {
      await playbackApi.seekTo(time)
      set({ currentTime: time })
    } catch (error) {
      console.error('Failed to seek:', error)
    }
  },

  toggleVocal: async (isVocal: boolean) => {
    try {
      await playbackApi.toggleVocal(isVocal)
      set({ isVocal })
    } catch (error) {
      console.error('Failed to toggle vocal:', error)
    }
  },

  setPitch: async (semitones: number) => {
    try {
      await playbackApi.setPitch(semitones)
      set({ pitch: semitones })
    } catch (error) {
      console.error('Failed to set pitch:', error)
    }
  },

  setSpeed: async (speed: number) => {
    try {
      await playbackApi.setSpeed(speed)
      set({ speed })
    } catch (error) {
      console.error('Failed to set speed:', error)
    }
  },

  updateState: async () => {
    try {
      const state = await playbackApi.getPlaybackState()
      set(state)
    } catch (error) {
      console.error('Failed to update playback state:', error)
    }
  },

  setCurrentTime: (time: number) => {
    set({ currentTime: time })
  },
}))
