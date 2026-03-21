import { create } from 'zustand'
import type { Song, PlaybackState } from '@/types'
import { playbackApi, interludeApi } from '@/lib/api'

interface PlaybackStore extends PlaybackState {
  currentSong: Song | null
  continuousPlay: boolean // 连播开关
  volume: number // 音量 0-1

  play: (song: Song) => Promise<void>
  pause: () => Promise<void>
  resume: () => Promise<void>
  stop: () => Promise<void>
  seek: (time: number) => Promise<void>
  toggleVocal: (isVocal: boolean) => Promise<void>
  setPitch: (semitones: number) => Promise<void>
  setSpeed: (speed: number) => Promise<void>
  setVolume: (volume: number) => Promise<void>
  setCurrentTime: (time: number) => void
  setDuration: (duration: number) => void
  setContinuousPlay: (enabled: boolean) => void
  onSongEnded: () => Promise<void>
  syncState: () => Promise<void>
}

export const usePlaybackStore = create<PlaybackStore>((set, get) => ({
  status: 'idle',
  currentSongId: null,
  currentTime: 0,
  duration: 0,
  isVocal: true,
  pitch: 0,
  speed: 1.0,
  volume: 0.8,
  currentSong: null,
  continuousPlay: true, // 默认开启连播

  play: async (song: Song) => {
    console.log('=== playbackStore.play ===')
    console.log('传入的 song:', song)

    // 先停止过场音乐
    try {
      await interludeApi.stopInterlude()
    } catch (e) {
      // 忽略错误
    }

    // 调用后端播放歌曲
    await playbackApi.playSong(song.id)

    set({
      status: 'playing',
      currentSongId: song.id,
      currentSong: song,
      duration: song.duration || 0,
      currentTime: 0,
    })
  },

  pause: async () => {
    await playbackApi.pauseSong()
    set({ status: 'paused' })
  },

  resume: async () => {
    await playbackApi.resumeSong()
    set({ status: 'playing' })
  },

  stop: async () => {
    await playbackApi.stopSong()
    set({
      status: 'idle',
      currentSongId: null,
      currentSong: null,
      currentTime: 0,
      duration: 0,
    })

    // 歌曲停止后，自动开始过场音乐
    try {
      await interludeApi.playInterlude()
      console.log('[播放] 已自动开始过场音乐')
    } catch (e) {
      console.log('[播放] 无过场音乐或启动失败')
    }
  },

  // 歌曲播放完成（自动结束）
  onSongEnded: async () => {
    const { continuousPlay } = get()

    set({
      status: 'idle',
      currentSongId: null,
      currentSong: null,
      currentTime: 0,
      duration: 0,
    })

    if (continuousPlay) {
      // 连播模式：不播放过场音乐，由队列处理下一首
      console.log('[播放] 连播模式，等待下一首')
    } else {
      // 非连播模式：等待5秒后播放过场音乐
      console.log('[播放] 非连播模式，5秒后播放过场音乐')
      setTimeout(async () => {
        // 检查是否有歌曲正在播放（可能用户已经手动播放了）
        const currentStatus = get().status
        if (currentStatus === 'idle') {
          try {
            await interludeApi.playInterlude()
            console.log('[播放] 已自动开始过场音乐')
          } catch (e) {
            console.log('[播放] 无过场音乐或启动失败')
          }
        }
      }, 5000)
    }
  },

  seek: async (time: number) => {
    await playbackApi.seekTo(time)
    set({ currentTime: time })
  },

  toggleVocal: async (isVocal: boolean) => {
    await playbackApi.toggleVocal(isVocal)
    set({ isVocal })
  },

  setPitch: async (semitones: number) => {
    await playbackApi.setPitch(semitones)
    set({ pitch: semitones })
  },

  setSpeed: async (speed: number) => {
    await playbackApi.setSpeed(speed)
    set({ speed })
  },

  setVolume: async (volume: number) => {
    await playbackApi.setVolume(volume)
    set({ volume })
  },

  setCurrentTime: (time: number) => {
    set({ currentTime: time })
  },

  setDuration: (duration: number) => {
    set({ duration })
  },

  setContinuousPlay: (enabled: boolean) => {
    set({ continuousPlay: enabled })
  },

  syncState: async () => {
    try {
      const state = await playbackApi.getPlaybackState()
      set({
        status: state.status,
        currentSongId: state.currentSongId,
        currentTime: state.currentTime,
        duration: state.duration,
        isVocal: state.isVocal,
        pitch: state.pitch,
        speed: state.speed,
        volume: state.volume,
      })
    } catch (e) {
      console.error('Failed to sync playback state:', e)
    }
  },
}))
