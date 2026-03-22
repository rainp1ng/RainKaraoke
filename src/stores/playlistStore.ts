import { create } from 'zustand'
import type { Playlist, PlaylistSong } from '@/types'
import { playlistApi } from '@/lib/api'

interface PlaylistState {
  playlists: Playlist[]
  currentPlaylist: Playlist | null
  currentPlaylistSongs: PlaylistSong[]
  isLoading: boolean
  error: string | null

  loadPlaylists: () => Promise<void>
  createPlaylist: (name: string, description?: string) => Promise<number>
  updatePlaylist: (id: number, name?: string, description?: string) => Promise<boolean>
  deletePlaylist: (id: number) => Promise<boolean>
  loadPlaylistSongs: (playlistId: number) => Promise<void>
  addSongToPlaylist: (playlistId: number, songId: number) => Promise<boolean>
  addSongsToPlaylist: (playlistId: number, songIds: number[]) => Promise<number>
  removeSongFromPlaylist: (playlistId: number, songId: number) => Promise<boolean>
  movePlaylistSong: (playlistId: number, songId: number, newPosition: number) => Promise<boolean>
  clearPlaylist: (playlistId: number) => Promise<boolean>
  clearError: () => void
}

export const usePlaylistStore = create<PlaylistState>((set, get) => ({
  playlists: [],
  currentPlaylist: null,
  currentPlaylistSongs: [],
  isLoading: false,
  error: null,

  loadPlaylists: async () => {
    set({ isLoading: true })
    try {
      const playlists = await playlistApi.getPlaylists()
      set({ playlists, isLoading: false })
    } catch (error) {
      console.error('Failed to load playlists:', error)
      set({ isLoading: false, error: String(error) })
    }
  },

  createPlaylist: async (name: string, description?: string) => {
    try {
      const id = await playlistApi.createPlaylist(name, description)
      await get().loadPlaylists()
      return id
    } catch (error) {
      console.error('Failed to create playlist:', error)
      set({ error: String(error) })
      return -1
    }
  },

  updatePlaylist: async (id: number, name?: string, description?: string) => {
    try {
      const result = await playlistApi.updatePlaylist(id, name, description)
      await get().loadPlaylists()
      return result
    } catch (error) {
      console.error('Failed to update playlist:', error)
      set({ error: String(error) })
      return false
    }
  },

  deletePlaylist: async (id: number) => {
    try {
      await playlistApi.deletePlaylist(id)
      set((state) => ({
        playlists: state.playlists.filter((p) => p.id !== id),
        currentPlaylist: state.currentPlaylist?.id === id ? null : state.currentPlaylist,
        currentPlaylistSongs: state.currentPlaylist?.id === id ? [] : state.currentPlaylistSongs,
      }))
      return true
    } catch (error) {
      console.error('Failed to delete playlist:', error)
      set({ error: String(error) })
      return false
    }
  },

  loadPlaylistSongs: async (playlistId: number) => {
    set({ isLoading: true })
    try {
      const [songs, playlist] = await Promise.all([
        playlistApi.getPlaylistSongs(playlistId),
        playlistApi.getPlaylistById(playlistId),
      ])
      set({
        currentPlaylistSongs: songs,
        currentPlaylist: playlist,
        isLoading: false,
      })
    } catch (error) {
      console.error('Failed to load playlist songs:', error)
      set({ isLoading: false, error: String(error) })
    }
  },

  addSongToPlaylist: async (playlistId: number, songId: number) => {
    try {
      await playlistApi.addSongToPlaylist(playlistId, songId)
      // 重新加载歌单歌曲
      if (get().currentPlaylist?.id === playlistId) {
        await get().loadPlaylistSongs(playlistId)
      }
      // 更新歌单列表（歌曲数量变化）
      await get().loadPlaylists()
      return true
    } catch (error) {
      console.error('Failed to add song to playlist:', error)
      set({ error: String(error) })
      return false
    }
  },

  addSongsToPlaylist: async (playlistId: number, songIds: number[]) => {
    try {
      const added = await playlistApi.addSongsToPlaylist(playlistId, songIds)
      // 重新加载歌单歌曲
      if (get().currentPlaylist?.id === playlistId) {
        await get().loadPlaylistSongs(playlistId)
      }
      // 更新歌单列表（歌曲数量变化）
      await get().loadPlaylists()
      return added
    } catch (error) {
      console.error('Failed to add songs to playlist:', error)
      set({ error: String(error) })
      return 0
    }
  },

  removeSongFromPlaylist: async (playlistId: number, songId: number) => {
    try {
      await playlistApi.removeSongFromPlaylist(playlistId, songId)
      // 更新当前歌单歌曲列表
      set((state) => ({
        currentPlaylistSongs: state.currentPlaylistSongs.filter((s) => s.songId !== songId),
      }))
      // 更新歌单列表（歌曲数量变化）
      await get().loadPlaylists()
      return true
    } catch (error) {
      console.error('Failed to remove song from playlist:', error)
      set({ error: String(error) })
      return false
    }
  },

  movePlaylistSong: async (playlistId: number, songId: number, newPosition: number) => {
    try {
      await playlistApi.movePlaylistSong(playlistId, songId, newPosition)
      // 重新加载歌单歌曲以获取新的排序
      if (get().currentPlaylist?.id === playlistId) {
        await get().loadPlaylistSongs(playlistId)
      }
      return true
    } catch (error) {
      console.error('Failed to move playlist song:', error)
      set({ error: String(error) })
      return false
    }
  },

  clearPlaylist: async (playlistId: number) => {
    try {
      await playlistApi.clearPlaylist(playlistId)
      set({
        currentPlaylistSongs: get().currentPlaylist?.id === playlistId ? [] : get().currentPlaylistSongs,
      })
      await get().loadPlaylists()
      return true
    } catch (error) {
      console.error('Failed to clear playlist:', error)
      set({ error: String(error) })
      return false
    }
  },

  clearError: () => {
    set({ error: null })
  },
}))
