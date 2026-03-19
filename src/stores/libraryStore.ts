import { create } from 'zustand'
import type { Song, Tag } from '@/types'
import { libraryApi } from '@/lib/api'

interface LibraryState {
  // 状态
  songs: Song[]
  totalCount: number
  currentPage: number
  pageSize: number
  isLoading: boolean
  error: string | null

  // 筛选条件
  searchQuery: string
  selectedArtist: string | null
  selectedGenre: string | null
  selectedLanguage: string | null
  sortBy: string
  sortOrder: 'asc' | 'desc'

  // 辅助数据
  artists: string[]
  genres: string[]
  languages: string[]

  // 操作
  loadSongs: () => Promise<void>
  loadArtists: () => Promise<void>
  loadGenres: () => Promise<void>
  loadLanguages: () => Promise<void>
  setSearchQuery: (query: string) => void
  setSelectedArtist: (artist: string | null) => void
  setSelectedGenre: (genre: string | null) => void
  setSelectedLanguage: (language: string | null) => void
  setSorting: (sortBy: string, sortOrder: 'asc' | 'desc') => void
  setPage: (page: number) => void
  setPageSize: (size: number) => void
  importFolder: (directory: string, recursive?: boolean) => Promise<{ success: number; skipped: number; failed: number }>
  importFile: (filePath: string) => Promise<number>
  deleteSong: (id: number) => Promise<void>
  clearFilters: () => void
}

export const useLibraryStore = create<LibraryState>((set, get) => ({
  // 初始状态
  songs: [],
  totalCount: 0,
  currentPage: 1,
  pageSize: 50,
  isLoading: false,
  error: null,

  searchQuery: '',
  selectedArtist: null,
  selectedGenre: null,
  selectedLanguage: null,
  sortBy: 'title',
  sortOrder: 'asc',

  artists: [],
  genres: [],
  languages: [],

  // 加载歌曲列表
  loadSongs: async () => {
    const state = get()
    set({ isLoading: true, error: null })

    try {
      const [songs, count] = await Promise.all([
        libraryApi.getSongs({
          page: state.currentPage,
          pageSize: state.pageSize,
          search: state.searchQuery || undefined,
          artist: state.selectedArtist || undefined,
          genre: state.selectedGenre || undefined,
          language: state.selectedLanguage || undefined,
          sortBy: state.sortBy,
          sortOrder: state.sortOrder,
        }),
        libraryApi.getSongsCount({
          search: state.searchQuery || undefined,
          artist: state.selectedArtist || undefined,
          genre: state.selectedGenre || undefined,
          language: state.selectedLanguage || undefined,
        }),
      ])

      set({ songs, totalCount: count, isLoading: false })
    } catch (error) {
      set({ error: String(error), isLoading: false })
    }
  },

  // 加载歌手列表
  loadArtists: async () => {
    try {
      const artists = await libraryApi.getArtists()
      set({ artists })
    } catch (error) {
      console.error('Failed to load artists:', error)
    }
  },

  // 加载风格列表
  loadGenres: async () => {
    try {
      const genres = await libraryApi.getGenres()
      set({ genres })
    } catch (error) {
      console.error('Failed to load genres:', error)
    }
  },

  // 加载语言列表
  loadLanguages: async () => {
    try {
      const languages = await libraryApi.getLanguages()
      set({ languages })
    } catch (error) {
      console.error('Failed to load languages:', error)
    }
  },

  // 设置搜索查询
  setSearchQuery: (query: string) => {
    set({ searchQuery: query, currentPage: 1 })
    get().loadSongs()
  },

  // 设置选中歌手
  setSelectedArtist: (artist: string | null) => {
    set({ selectedArtist: artist, currentPage: 1 })
    get().loadSongs()
  },

  // 设置选中风格
  setSelectedGenre: (genre: string | null) => {
    set({ selectedGenre: genre, currentPage: 1 })
    get().loadSongs()
  },

  // 设置选中语言
  setSelectedLanguage: (language: string | null) => {
    set({ selectedLanguage: language, currentPage: 1 })
    get().loadSongs()
  },

  // 设置排序
  setSorting: (sortBy: string, sortOrder: 'asc' | 'desc') => {
    set({ sortBy, sortOrder })
    get().loadSongs()
  },

  // 设置页码
  setPage: (page: number) => {
    set({ currentPage: page })
    get().loadSongs()
  },

  // 设置每页数量
  setPageSize: (size: number) => {
    set({ pageSize: size, currentPage: 1 })
    get().loadSongs()
  },

  // 导入文件夹
  importFolder: async (directory: string, recursive: boolean = true) => {
    set({ isLoading: true, error: null })
    try {
      const result = await libraryApi.importSongs(directory, recursive)
      await get().loadSongs()
      await Promise.all([
        get().loadArtists(),
        get().loadGenres(),
        get().loadLanguages(),
      ])
      set({ isLoading: false })
      return result
    } catch (error) {
      set({ error: String(error), isLoading: false })
      throw error
    }
  },

  // 导入单个文件
  importFile: async (filePath: string) => {
    try {
      const id = await libraryApi.importSingleFile(filePath)
      await get().loadSongs()
      return id
    } catch (error) {
      set({ error: String(error) })
      throw error
    }
  },

  // 删除歌曲
  deleteSong: async (id: number) => {
    try {
      await libraryApi.deleteSong(id)
      set((state) => ({
        songs: state.songs.filter((s) => s.id !== id),
        totalCount: state.totalCount - 1,
      }))
    } catch (error) {
      set({ error: String(error) })
      throw error
    }
  },

  // 清除筛选条件
  clearFilters: () => {
    set({
      searchQuery: '',
      selectedArtist: null,
      selectedGenre: null,
      selectedLanguage: null,
      currentPage: 1,
    })
    get().loadSongs()
  },
}))
