import { useEffect, useState } from 'react'
import { open, confirm } from '@tauri-apps/plugin-dialog'
import { Search, Plus, FolderOpen, Trash2, X, Loader2, MoreVertical, Music, FileText, Edit2 } from 'lucide-react'
import { useLibraryStore, useQueueStore } from '@/stores'
import { formatDuration } from '@/utils/format'
import { libraryApi } from '@/lib/api'
import type { Song } from '@/types'

interface EditFormData {
  title: string
  artist: string
  album: string
}

function Library() {
  const {
    songs,
    totalCount,
    currentPage,
    pageSize,
    isLoading,
    error,
    searchQuery,
    selectedArtist,
    selectedGenre,
    artists,
    loadSongs,
    loadArtists,
    loadGenres,
    setSearchQuery,
    setSelectedArtist,
    setPage,
    importFolder,
    deleteSong,
    clearFilters,
  } = useLibraryStore()

  const { addToQueue, error: queueError, clearError } = useQueueStore()

  const [importing, setImporting] = useState(false)
  const [importResult, setImportResult] = useState<{ success: number; skipped: number; failed: number } | null>(null)
  const [selectedSongs, setSelectedSongs] = useState<Set<number>>(new Set())
  const [menuOpenId, setMenuOpenId] = useState<number | null>(null)
  const [editingSong, setEditingSong] = useState<Song | null>(null)
  const [editForm, setEditForm] = useState<EditFormData>({ title: '', artist: '', album: '' })
  const [saving, setSaving] = useState(false)

  // 初始加载
  useEffect(() => {
    loadSongs()
    loadArtists()
    loadGenres()
  }, [])

  // 处理导入文件夹
  const handleImportFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择歌曲文件夹',
      })

      if (selected) {
        setImporting(true)
        setImportResult(null)
        const result = await importFolder(selected as string, true)
        setImportResult(result)
      }
    } catch (err) {
      console.error('导入失败:', err)
    } finally {
      setImporting(false)
    }
  }

  // 处理搜索
  const handleSearch = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    loadSongs()
  }

  // 处理添加到队列
  const handleAddToQueue = async (songId: number) => {
    const success = await addToQueue(songId)
    if (!success) {
      // 3秒后自动清除错误
      setTimeout(() => clearError(), 3000)
    }
  }

  // 处理批量添加
  const handleAddSelectedToQueue = async () => {
    for (const songId of selectedSongs) {
      await addToQueue(songId)
    }
    setSelectedSongs(new Set())
  }

  // 处理删除
  const handleDelete = async (song: Song) => {
    const confirmed = await confirm(`确定要删除 "${song.title}" 吗？`, {
      title: '确认删除',
      kind: 'warning',
    })
    if (confirmed) {
      await deleteSong(song.id)
    }
  }

  // 切换选择
  const toggleSelect = (songId: number) => {
    const newSelected = new Set(selectedSongs)
    if (newSelected.has(songId)) {
      newSelected.delete(songId)
    } else {
      newSelected.add(songId)
    }
    setSelectedSongs(newSelected)
  }

  // 导入原唱
  const handleImportVocal = async (song: Song) => {
    const selected = await open({
      multiple: false,
      title: '选择原唱音频文件',
      filters: [
        { name: '音频文件', extensions: ['mp3', 'flac', 'wav', 'ogg', 'm4a', 'aac'] }
      ]
    })
    if (selected) {
      try {
        await libraryApi.importVocal(song.id, selected as string)
        loadSongs()
        setMenuOpenId(null)
      } catch (err) {
        console.error('导入原唱失败:', err)
      }
    }
  }

  // 导入歌词
  const handleImportLyrics = async (song: Song) => {
    const selected = await open({
      multiple: false,
      title: '选择歌词文件',
      filters: [
        { name: '歌词文件', extensions: ['lrc', 'ksc', 'txt'] }
      ]
    })
    if (selected) {
      try {
        await libraryApi.importLyrics(song.id, selected as string)
        loadSongs()
        setMenuOpenId(null)
      } catch (err) {
        console.error('导入歌词失败:', err)
      }
    }
  }

  // 打开编辑对话框
  const handleOpenEdit = (song: Song) => {
    setEditingSong(song)
    setEditForm({
      title: song.title || '',
      artist: song.artist || '',
      album: song.album || '',
    })
    setMenuOpenId(null)
  }

  // 保存编辑
  const handleSaveEdit = async () => {
    if (!editingSong) return

    setSaving(true)
    try {
      await libraryApi.updateSongMetadata(
        editingSong.id,
        editForm.title || undefined,
        editForm.artist || undefined,
        editForm.album || undefined
      )
      setEditingSong(null)
      loadSongs()
    } catch (err) {
      console.error('保存失败:', err)
    } finally {
      setSaving(false)
    }
  }

  // 计算总页数
  const totalPages = Math.ceil(totalCount / pageSize)

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* 搜索栏 */}
      <div className="p-4 border-b border-dark-700 space-y-3">
        <form onSubmit={handleSearch} className="flex gap-2">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-dark-400" />
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="搜索歌曲、歌手..."
              className="w-full bg-dark-800 border border-dark-600 rounded-lg pl-10 pr-4 py-2 text-sm focus:outline-none focus:border-primary-500"
            />
          </div>
          <button
            type="button"
            onClick={handleImportFolder}
            disabled={importing}
            className="px-4 py-2 bg-primary-600 hover:bg-primary-700 disabled:bg-dark-600 rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
          >
            {importing ? <Loader2 className="w-4 h-4 animate-spin" /> : <FolderOpen className="w-4 h-4" />}
            {importing ? '导入中...' : '导入文件夹'}
          </button>
        </form>

        {/* 筛选标签 */}
        <div className="flex gap-2 flex-wrap items-center">
          <button
            onClick={() => setSelectedArtist(null)}
            className={`px-3 py-1 rounded-full text-sm transition-colors ${
              !selectedArtist ? 'bg-primary-600 text-white' : 'bg-dark-700 hover:bg-dark-600'
            }`}
          >
            全部歌手
          </button>
          {artists.slice(0, 10).map((artist) => (
            <button
              key={artist}
              onClick={() => setSelectedArtist(artist)}
              className={`px-3 py-1 rounded-full text-sm transition-colors ${
                selectedArtist === artist ? 'bg-primary-600 text-white' : 'bg-dark-700 hover:bg-dark-600'
              }`}
            >
              {artist}
            </button>
          ))}

          {(searchQuery || selectedArtist || selectedGenre) && (
            <button
              onClick={clearFilters}
              className="px-3 py-1 rounded-full text-sm bg-dark-700 hover:bg-dark-600 transition-colors flex items-center gap-1"
            >
              <X className="w-3 h-3" />
              清除筛选
            </button>
          )}
        </div>

        {/* 导入结果提示 */}
        {importResult && (
          <div className="bg-dark-800 rounded-lg p-3 text-sm">
            <span className="text-green-400">成功导入 {importResult.success} 首</span>
            {importResult.skipped > 0 && (
              <span className="text-yellow-400 ml-3">跳过 {importResult.skipped} 首（已存在）</span>
            )}
            {importResult.failed > 0 && (
              <span className="text-red-400 ml-3">失败 {importResult.failed} 首</span>
            )}
          </div>
        )}
      </div>

      {/* 错误提示 */}
      {error && (
        <div className="p-4 bg-red-900/50 text-red-300 text-sm">{error}</div>
      )}

      {/* 队列错误提示 */}
      {queueError && (
        <div className="p-4 bg-yellow-900/50 text-yellow-300 text-sm flex items-center justify-between">
          <span>{queueError}</span>
          <button onClick={clearError} className="hover:text-yellow-100">
            <X className="w-4 h-4" />
          </button>
        </div>
      )}

      {/* 歌曲列表 */}
      <div className="flex-1 overflow-y-auto">
        {isLoading ? (
          <div className="flex items-center justify-center h-64">
            <Loader2 className="w-8 h-8 animate-spin text-primary-500" />
          </div>
        ) : songs.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-64 text-dark-400">
            <FolderOpen className="w-12 h-12 mb-2" />
            <p>暂无歌曲</p>
            <p className="text-sm mt-1">点击上方"导入文件夹"添加歌曲</p>
          </div>
        ) : (
          <table className="w-full">
            <thead className="bg-dark-800 text-left text-sm text-dark-400 sticky top-0">
              <tr>
                <th className="px-4 py-2 w-10">
                  <input
                    type="checkbox"
                    checked={selectedSongs.size === songs.length && songs.length > 0}
                    onChange={(e) => {
                      if (e.target.checked) {
                        setSelectedSongs(new Set(songs.map((s) => s.id)))
                      } else {
                        setSelectedSongs(new Set())
                      }
                    }}
                    className="rounded"
                  />
                </th>
                <th className="px-4 py-2">#</th>
                <th className="px-4 py-2">歌曲名</th>
                <th className="px-4 py-2">歌手</th>
                <th className="px-4 py-2">专辑</th>
                <th className="px-4 py-2 w-20">时长</th>
                <th className="px-4 py-2 w-24">音轨</th>
                <th className="px-4 py-2 w-20">操作</th>
              </tr>
            </thead>
            <tbody>
              {songs.map((song, index) => (
                <tr
                  key={song.id}
                  className={`border-b border-dark-800 hover:bg-dark-800/50 transition-colors ${
                    selectedSongs.has(song.id) ? 'bg-primary-900/20' : ''
                  }`}
                >
                  <td className="px-4 py-2">
                    <input
                      type="checkbox"
                      checked={selectedSongs.has(song.id)}
                      onChange={() => toggleSelect(song.id)}
                      className="rounded"
                    />
                  </td>
                  <td className="px-4 py-2 text-dark-400">
                    {(currentPage - 1) * pageSize + index + 1}
                  </td>
                  <td className="px-4 py-2 font-medium">{song.title}</td>
                  <td className="px-4 py-2 text-dark-300">{song.artist || '-'}</td>
                  <td className="px-4 py-2 text-dark-400">{song.album || '-'}</td>
                  <td className="px-4 py-2 text-dark-400">
                    {song.duration ? formatDuration(song.duration) : '-'}
                  </td>
                  <td className="px-4 py-2">
                    <div className="flex gap-1">
                      {song.hasVocal && (
                        <span className="px-1.5 py-0.5 bg-green-900/50 text-green-400 rounded text-xs">
                          原唱
                        </span>
                      )}
                      {song.hasInstrumental && (
                        <span className="px-1.5 py-0.5 bg-blue-900/50 text-blue-400 rounded text-xs">
                          伴奏
                        </span>
                      )}
                      {song.lyricsPath && (
                        <span className="px-1.5 py-0.5 bg-purple-900/50 text-purple-400 rounded text-xs">
                          歌词
                        </span>
                      )}
                    </div>
                  </td>
                  <td className="px-4 py-2">
                    <div className="flex gap-1 items-center relative">
                      <button
                        onClick={() => handleAddToQueue(song.id)}
                        className="p-1 hover:bg-dark-600 rounded transition-colors"
                        title="添加到队列"
                      >
                        <Plus className="w-4 h-4" />
                      </button>
                      <button
                        onClick={() => setMenuOpenId(menuOpenId === song.id ? null : song.id)}
                        className="p-1 hover:bg-dark-600 rounded transition-colors"
                        title="更多操作"
                      >
                        <MoreVertical className="w-4 h-4" />
                      </button>
                      {menuOpenId === song.id && (
                        <div className="absolute right-0 top-full mt-1 bg-dark-800 border border-dark-600 rounded shadow-lg z-10 min-w-[140px]">
                          <button
                            onClick={() => handleOpenEdit(song)}
                            className="w-full px-3 py-2 text-left text-sm hover:bg-dark-700 flex items-center gap-2"
                          >
                            <Edit2 className="w-4 h-4" />
                            编辑信息
                          </button>
                          <button
                            onClick={() => handleImportVocal(song)}
                            className="w-full px-3 py-2 text-left text-sm hover:bg-dark-700 flex items-center gap-2"
                          >
                            <Music className="w-4 h-4" />
                            导入原唱
                          </button>
                          <button
                            onClick={() => handleImportLyrics(song)}
                            className="w-full px-3 py-2 text-left text-sm hover:bg-dark-700 flex items-center gap-2"
                          >
                            <FileText className="w-4 h-4" />
                            导入歌词
                          </button>
                          <hr className="border-dark-600" />
                          <button
                            onClick={() => {
                              handleDelete(song)
                              setMenuOpenId(null)
                            }}
                            className="w-full px-3 py-2 text-left text-sm hover:bg-dark-700 text-red-400 flex items-center gap-2"
                          >
                            <Trash2 className="w-4 h-4" />
                            删除歌曲
                          </button>
                        </div>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* 底部工具栏 */}
      {songs.length > 0 && (
        <div className="p-3 border-t border-dark-700 flex items-center justify-between">
          <div className="flex items-center gap-4">
            <span className="text-sm text-dark-400">
              共 {totalCount} 首歌曲
            </span>
            {selectedSongs.size > 0 && (
              <button
                onClick={handleAddSelectedToQueue}
                className="px-3 py-1 bg-primary-600 hover:bg-primary-700 rounded text-sm transition-colors"
              >
                添加 {selectedSongs.size} 首到队列
              </button>
            )}
          </div>

          {/* 分页 */}
          {totalPages > 1 && (
            <div className="flex items-center gap-2">
              <button
                onClick={() => setPage(currentPage - 1)}
                disabled={currentPage === 1}
                className="px-3 py-1 bg-dark-700 hover:bg-dark-600 disabled:opacity-50 disabled:cursor-not-allowed rounded text-sm transition-colors"
              >
                上一页
              </button>
              <span className="text-sm text-dark-400">
                {currentPage} / {totalPages}
              </span>
              <button
                onClick={() => setPage(currentPage + 1)}
                disabled={currentPage === totalPages}
                className="px-3 py-1 bg-dark-700 hover:bg-dark-600 disabled:opacity-50 disabled:cursor-not-allowed rounded text-sm transition-colors"
              >
                下一页
              </button>
            </div>
          )}
        </div>
      )}

      {/* 编辑歌曲信息对话框 */}
      {editingSong && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50">
          <div className="bg-dark-800 rounded-lg p-6 w-full max-w-md mx-4">
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-lg font-medium">编辑歌曲信息</h3>
              <button
                onClick={() => setEditingSong(null)}
                className="p-1 hover:bg-dark-600 rounded transition-colors"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="block text-sm text-dark-300 mb-1">歌曲名</label>
                <input
                  type="text"
                  value={editForm.title}
                  onChange={(e) => setEditForm({ ...editForm, title: e.target.value })}
                  className="w-full bg-dark-700 border border-dark-600 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-primary-500"
                  placeholder="输入歌曲名"
                />
              </div>

              <div>
                <label className="block text-sm text-dark-300 mb-1">歌手</label>
                <input
                  type="text"
                  value={editForm.artist}
                  onChange={(e) => setEditForm({ ...editForm, artist: e.target.value })}
                  className="w-full bg-dark-700 border border-dark-600 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-primary-500"
                  placeholder="输入歌手名"
                />
              </div>

              <div>
                <label className="block text-sm text-dark-300 mb-1">专辑</label>
                <input
                  type="text"
                  value={editForm.album}
                  onChange={(e) => setEditForm({ ...editForm, album: e.target.value })}
                  className="w-full bg-dark-700 border border-dark-600 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-primary-500"
                  placeholder="输入专辑名"
                />
              </div>
            </div>

            <div className="mt-6 flex justify-end gap-3">
              <button
                onClick={() => setEditingSong(null)}
                className="px-4 py-2 bg-dark-700 hover:bg-dark-600 rounded-lg text-sm transition-colors"
              >
                取消
              </button>
              <button
                onClick={handleSaveEdit}
                disabled={saving || !editForm.title.trim()}
                className="px-4 py-2 bg-primary-600 hover:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
              >
                {saving && <Loader2 className="w-4 h-4 animate-spin" />}
                {saving ? '保存中...' : '保存'}
              </button>
            </div>

            <p className="mt-4 text-xs text-dark-500 text-center">
              修改将同步更新到音频文件元数据
            </p>
          </div>
        </div>
      )}
    </div>
  )
}

export default Library
