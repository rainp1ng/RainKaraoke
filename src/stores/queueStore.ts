import { create } from 'zustand'
import type { QueueItem } from '@/types'
import { queueApi, libraryApi } from '@/lib/api'

interface QueueState {
  items: QueueItem[]
  isLoading: boolean
  error: string | null

  loadQueue: () => Promise<void>
  addToQueue: (songId: number) => Promise<boolean>
  addManyToQueue: (songIds: number[]) => Promise<void>
  removeFromQueue: (queueId: number) => Promise<void>
  moveItem: (queueId: number, newPosition: number) => Promise<void>
  moveToTop: (queueId: number) => Promise<void>
  moveToNext: (queueId: number, currentSongId: number) => Promise<void>
  clearQueue: () => Promise<void>
  playNext: () => Promise<void>
  clearError: () => void
}

export const useQueueStore = create<QueueState>((set, get) => ({
  items: [],
  isLoading: false,
  error: null,

  loadQueue: async () => {
    set({ isLoading: true })
    try {
      const items = await queueApi.getQueue()

      // 加载歌曲详情
      const itemsWithSongs = await Promise.all(
        items.map(async (item) => {
          const song = await libraryApi.getSongById(item.songId)
          return { ...item, song: song || undefined }
        })
      )

      set({ items: itemsWithSongs, isLoading: false })
    } catch (error) {
      console.error('Failed to load queue:', error)
      set({ isLoading: false })
    }
  },

  addToQueue: async (songId: number) => {
    try {
      await queueApi.addToQueue(songId)
      await get().loadQueue()
      set({ error: null })
      return true
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error)
      set({ error: message })
      return false
    }
  },

  clearError: () => {
    set({ error: null })
  },

  addManyToQueue: async (songIds: number[]) => {
    try {
      for (const songId of songIds) {
        await queueApi.addToQueue(songId)
      }
      await get().loadQueue()
    } catch (error) {
      console.error('Failed to add many to queue:', error)
    }
  },

  removeFromQueue: async (queueId: number) => {
    try {
      await queueApi.removeFromQueue(queueId)
      set((state) => ({
        items: state.items.filter((item) => item.id !== queueId),
      }))
    } catch (error) {
      console.error('Failed to remove from queue:', error)
    }
  },

  moveItem: async (queueId: number, newPosition: number) => {
    try {
      await queueApi.moveQueueItem(queueId, newPosition)
      await get().loadQueue()
    } catch (error) {
      console.error('Failed to move queue item:', error)
    }
  },

  moveToTop: async (queueId: number) => {
    try {
      await queueApi.moveToTop(queueId)
      await get().loadQueue()
    } catch (error) {
      console.error('Failed to move to top:', error)
    }
  },

  moveToNext: async (queueId: number, currentSongId: number) => {
    try {
      await queueApi.moveToNext(queueId, currentSongId)
      await get().loadQueue()
    } catch (error) {
      console.error('Failed to move to next:', error)
    }
  },

  clearQueue: async () => {
    try {
      await queueApi.clearQueue()
      set({ items: [] })
    } catch (error) {
      console.error('Failed to clear queue:', error)
    }
  },

  playNext: async () => {
    try {
      await queueApi.playNext()
      await get().loadQueue()
    } catch (error) {
      console.error('Failed to play next:', error)
    }
  },
}))
