import { useEffect, useState } from 'react'
import { confirm } from '@tauri-apps/plugin-dialog'
import {
  ListMusic, Plus, Trash2, MoreVertical, Play, GripVertical,
  Edit2, X, Music, ChevronRight, ListPlus, ChevronUp, ChevronDown
} from 'lucide-react'
import { usePlaylistStore, usePlaybackStore, useQueueStore } from '@/stores'
import { formatDuration } from '@/utils/format'
import type { Playlist, PlaylistSong } from '@/types'

function PlaylistPanel() {
  const {
    playlists,
    currentPlaylist,
    currentPlaylistSongs,
    isLoading,
    loadPlaylists,
    createPlaylist,
    updatePlaylist,
    deletePlaylist,
    loadPlaylistSongs,
    removeSongFromPlaylist,
    movePlaylistSong,
    clearPlaylist,
  } = usePlaylistStore()

  const { play } = usePlaybackStore()
  const { addToQueue } = useQueueStore()

  const [showCreateModal, setShowCreateModal] = useState(false)
  const [newPlaylistName, setNewPlaylistName] = useState('')
  const [newPlaylistDesc, setNewPlaylistDesc] = useState('')
  const [editingPlaylist, setEditingPlaylist] = useState<Playlist | null>(null)
  const [editName, setEditName] = useState('')
  const [editDesc, setEditDesc] = useState('')
  const [draggedSongId, setDraggedSongId] = useState<number | null>(null)
  const [dragOverIndex, setDragOverIndex] = useState<number | null>(null)
  const [menuOpenId, setMenuOpenId] = useState<number | null>(null)
  const [isMoving, setIsMoving] = useState(false)

  useEffect(() => {
    loadPlaylists()
  }, [])

  // 创建歌单
  const handleCreatePlaylist = async () => {
    if (!newPlaylistName.trim()) return

    const id = await createPlaylist(newPlaylistName.trim(), newPlaylistDesc.trim() || undefined)
    if (id > 0) {
      setShowCreateModal(false)
      setNewPlaylistName('')
      setNewPlaylistDesc('')
    }
  }

  // 删除歌单
  const handleDeletePlaylist = async (playlist: Playlist) => {
    const confirmed = await confirm(`确定要删除歌单 "${playlist.name}" 吗？`, {
      title: '确认删除',
      kind: 'warning',
    })
    if (confirmed) {
      await deletePlaylist(playlist.id)
    }
  }

  // 更新歌单
  const handleUpdatePlaylist = async () => {
    if (!editingPlaylist || !editName.trim()) return

    await updatePlaylist(editingPlaylist.id, editName.trim(), editDesc.trim() || undefined)
    setEditingPlaylist(null)
  }

  // 选择歌单查看
  const handleSelectPlaylist = async (playlist: Playlist) => {
    console.log('选择歌单:', playlist)
    try {
      await loadPlaylistSongs(playlist.id)
      console.log('加载成功')
    } catch (error) {
      console.error('加载歌单失败:', error)
    }
  }

  // 返回歌单列表
  const handleBackToList = () => {
    usePlaylistStore.setState({ currentPlaylist: null, currentPlaylistSongs: [] })
  }

  // 播放歌单中的歌曲
  const handlePlaySong = async (playlistSong: PlaylistSong) => {
    // 需要通过 songId 获取完整的 Song 对象
    const { libraryApi } = await import('@/lib/api')
    const fullSong = await libraryApi.getSongById(playlistSong.songId)
    if (fullSong) {
      await play(fullSong)
    }
  }

  // 播放全部歌曲
  const handlePlayAll = async () => {
    if (currentPlaylistSongs.length === 0 || !currentPlaylist) return

    try {
      const { libraryApi } = await import('@/lib/api')

      // 播放第一首歌曲
      const firstSong = await libraryApi.getSongById(currentPlaylistSongs[0].songId)
      if (firstSong) {
        await play(firstSong)
      }

      // 将其余歌曲添加到队列
      for (let i = 1; i < currentPlaylistSongs.length; i++) {
        await addToQueue(currentPlaylistSongs[i].songId)
      }
    } catch (error) {
      console.error('播放全部失败:', error)
    }
  }

  // 全部添加到队列
  const handleAddAllToQueue = async () => {
    if (currentPlaylistSongs.length === 0) return

    try {
      for (const song of currentPlaylistSongs) {
        await addToQueue(song.songId)
      }
    } catch (error) {
      console.error('添加到队列失败:', error)
    }
  }

  // 上移歌曲
  const handleMoveUp = async (index: number) => {
    if (index === 0 || !currentPlaylist || isMoving) return
    setIsMoving(true)
    const song = currentPlaylistSongs[index]
    await movePlaylistSong(currentPlaylist.id, song.songId, song.position - 1)
    setIsMoving(false)
  }

  // 下移歌曲
  const handleMoveDown = async (index: number) => {
    if (index === currentPlaylistSongs.length - 1 || !currentPlaylist || isMoving) return
    setIsMoving(true)
    const song = currentPlaylistSongs[index]
    await movePlaylistSong(currentPlaylist.id, song.songId, song.position + 1)
    setIsMoving(false)
  }

  // 添加到队列
  const handleAddToQueue = async (playlistSong: PlaylistSong) => {
    await addToQueue(playlistSong.songId)
  }

  // 从歌单移除
  const handleRemoveFromPlaylist = async (playlistSong: PlaylistSong) => {
    if (currentPlaylist) {
      await removeSongFromPlaylist(currentPlaylist.id, playlistSong.songId)
    }
  }

  // 拖拽开始
  const handleDragStart = (e: React.DragEvent, songId: number, index: number) => {
    setDraggedSongId(songId)
    e.dataTransfer.effectAllowed = 'move'
    e.dataTransfer.setData('text/plain', index.toString())
  }

  // 拖拽悬停
  const handleDragOver = (e: React.DragEvent, index: number) => {
    e.preventDefault()
    if (draggedSongId !== null) {
      setDragOverIndex(index)
    }
  }

  // 拖拽结束
  const handleDragEnd = () => {
    setDraggedSongId(null)
    setDragOverIndex(null)
  }

  // 放下
  const handleDrop = async (e: React.DragEvent, targetIndex: number) => {
    e.preventDefault()
    if (draggedSongId === null || !currentPlaylist) return

    const sourceIndex = parseInt(e.dataTransfer.getData('text/plain'))
    if (sourceIndex === targetIndex) {
      handleDragEnd()
      return
    }

    await movePlaylistSong(currentPlaylist.id, draggedSongId, targetIndex)
    handleDragEnd()
  }

  // 清空歌单
  const handleClearPlaylist = async () => {
    if (!currentPlaylist) return

    const confirmed = await confirm(`确定要清空歌单 "${currentPlaylist.name}" 吗？`, {
      title: '确认清空',
      kind: 'warning',
    })
    if (confirmed) {
      await clearPlaylist(currentPlaylist.id)
    }
  }

  // 正在查看歌单详情
  if (currentPlaylist) {
    return (
      <div className="flex flex-col h-full">
        {/* 歌单头部 */}
        <div className="flex items-center gap-3 p-4 border-b border-dark-700">
          <button
            onClick={handleBackToList}
            className="p-1 hover:bg-dark-700 rounded transition-colors"
          >
            <ChevronRight className="w-5 h-5 rotate-180" />
          </button>
          <div className="flex-1 min-w-0">
            <h2 className="font-semibold text-lg truncate">{currentPlaylist.name}</h2>
            <p className="text-sm text-dark-400">
              共 {currentPlaylistSongs.length} 首歌曲
            </p>
          </div>
          {currentPlaylistSongs.length > 0 && (
            <div className="flex items-center gap-1">
              <button
                onClick={handlePlayAll}
                className="px-3 py-1.5 bg-primary-600 hover:bg-primary-700 rounded text-sm transition-colors flex items-center gap-1"
                title="播放全部"
              >
                <Play className="w-4 h-4" />
                播放全部
              </button>
              <button
                onClick={handleAddAllToQueue}
                className="px-3 py-1.5 bg-dark-700 hover:bg-dark-600 rounded text-sm transition-colors flex items-center gap-1"
                title="全部添加到队列"
              >
                <ListPlus className="w-4 h-4" />
                添加到队列
              </button>
              <button
                onClick={handleClearPlaylist}
                className="p-2 hover:bg-dark-700 rounded text-dark-400 hover:text-red-400 transition-colors"
                title="清空歌单"
              >
                <Trash2 className="w-4 h-4" />
              </button>
            </div>
          )}
        </div>

        {/* 歌曲列表 */}
        <div className="flex-1 overflow-y-auto pb-4">
          {isLoading ? (
            <div className="flex items-center justify-center h-full text-dark-400">
              加载中...
            </div>
          ) : currentPlaylistSongs.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-dark-400">
              <Music className="w-12 h-12 mb-2 opacity-50" />
              <p>歌单为空</p>
              <p className="text-sm mt-1">从媒体库添加歌曲到这个歌单</p>
            </div>
          ) : (
            <div className="divide-y divide-dark-700 pb-2">
              {currentPlaylistSongs.map((song, index) => (
                <div
                  key={song.id}
                  draggable
                  onDragStart={(e) => handleDragStart(e, song.songId, index)}
                  onDragOver={(e) => handleDragOver(e, index)}
                  onDragEnd={handleDragEnd}
                  onDrop={(e) => handleDrop(e, index)}
                  className={`flex items-center gap-2 p-3 hover:bg-dark-800/50 transition-colors cursor-pointer group ${
                    dragOverIndex === index ? 'border-t-2 border-primary-500' : ''
                  }`}
                >
                  {/* 拖拽手柄 */}
                  <div className="opacity-0 group-hover:opacity-100 cursor-grab">
                    <GripVertical className="w-4 h-4 text-dark-500" />
                  </div>

                  {/* 序号 */}
                  <span className="w-6 text-sm text-dark-500 text-center">{index + 1}</span>

                  {/* 歌曲信息 */}
                  <div
                    className="flex-1 min-w-0"
                    onClick={() => handlePlaySong(song)}
                  >
                    <p className="truncate">{song.title || '未知歌曲'}</p>
                    <p className="text-sm text-dark-400 truncate">{song.artist || '未知歌手'}</p>
                  </div>

                  {/* 时长 */}
                  {song.duration && (
                    <span className="text-sm text-dark-400">
                      {formatDuration(song.duration)}
                    </span>
                  )}

                  {/* 操作按钮 */}
                  <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100">
                    <button
                      onClick={(e) => {
                        e.stopPropagation()
                        handleMoveUp(index)
                      }}
                      disabled={index === 0 || isMoving}
                      className="p-1 hover:bg-dark-600 rounded disabled:opacity-30 disabled:cursor-not-allowed"
                      title="上移"
                    >
                      <ChevronUp className="w-4 h-4" />
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation()
                        handleMoveDown(index)
                      }}
                      disabled={index === currentPlaylistSongs.length - 1 || isMoving}
                      className="p-1 hover:bg-dark-600 rounded disabled:opacity-30 disabled:cursor-not-allowed"
                      title="下移"
                    >
                      <ChevronDown className="w-4 h-4" />
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation()
                        handleAddToQueue(song)
                      }}
                      className="p-1 hover:bg-dark-600 rounded"
                      title="添加到播放队列"
                    >
                      <Plus className="w-4 h-4" />
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation()
                        handleRemoveFromPlaylist(song)
                      }}
                      className="p-1 hover:bg-dark-600 rounded text-dark-400 hover:text-red-400"
                      title="从歌单移除"
                    >
                      <X className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    )
  }

  // 歌单列表视图
  return (
    <div className="flex flex-col h-full">
      {/* 头部 */}
      <div className="flex items-center justify-between p-4 border-b border-dark-700">
        <h2 className="font-semibold text-lg">歌单</h2>
        <button
          onClick={() => setShowCreateModal(true)}
          className="flex items-center gap-1 px-3 py-1.5 bg-primary-600 hover:bg-primary-700 rounded text-sm transition-colors"
        >
          <Plus className="w-4 h-4" />
          新建歌单
        </button>
      </div>

      {/* 歌单列表 */}
      <div className="flex-1 overflow-y-auto">
        {isLoading ? (
          <div className="flex items-center justify-center h-full text-dark-400">
            加载中...
          </div>
        ) : playlists.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-dark-400">
            <ListMusic className="w-12 h-12 mb-2 opacity-50" />
            <p>暂无歌单</p>
            <p className="text-sm mt-1">点击上方按钮创建歌单</p>
          </div>
        ) : (
          <div className="divide-y divide-dark-700">
            {playlists.map((playlist) => (
              <div
                key={playlist.id}
                className="flex items-center gap-3 p-4 hover:bg-dark-800/50 transition-colors cursor-pointer group"
              >
                {/* 图标 */}
                <div className="w-10 h-10 bg-dark-700 rounded flex items-center justify-center">
                  <ListMusic className="w-5 h-5 text-primary-400" />
                </div>

                {/* 信息 */}
                <div className="flex-1 min-w-0" onClick={() => handleSelectPlaylist(playlist)}>
                  <p className="font-medium truncate">{playlist.name}</p>
                  <p className="text-sm text-dark-400">
                    {playlist.songCount} 首歌曲
                  </p>
                </div>

                {/* 菜单 */}
                <div className="relative opacity-0 group-hover:opacity-100">
                  <button
                    onClick={(e) => {
                      e.stopPropagation()
                      setMenuOpenId(menuOpenId === playlist.id ? null : playlist.id)
                    }}
                    className="p-1 hover:bg-dark-700 rounded"
                  >
                    <MoreVertical className="w-4 h-4" />
                  </button>

                  {menuOpenId === playlist.id && (
                    <div className="absolute right-0 top-full mt-1 bg-dark-800 border border-dark-600 rounded-lg shadow-lg py-1 min-w-[100px] z-10">
                      <button
                        onClick={(e) => {
                          e.stopPropagation()
                          setEditingPlaylist(playlist)
                          setEditName(playlist.name)
                          setEditDesc(playlist.description || '')
                          setMenuOpenId(null)
                        }}
                        className="w-full flex items-center gap-2 px-3 py-1.5 text-sm hover:bg-dark-700 text-left"
                      >
                        <Edit2 className="w-3.5 h-3.5" />
                        编辑
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation()
                          handleDeletePlaylist(playlist)
                          setMenuOpenId(null)
                        }}
                        className="w-full flex items-center gap-2 px-3 py-1.5 text-sm hover:bg-dark-700 text-left text-red-400"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                        删除
                      </button>
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* 创建歌单弹窗 */}
      {showCreateModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-dark-800 rounded-lg p-4 w-80">
            <h3 className="font-semibold mb-4">新建歌单</h3>
            <input
              type="text"
              value={newPlaylistName}
              onChange={(e) => setNewPlaylistName(e.target.value)}
              placeholder="歌单名称"
              className="w-full bg-dark-700 rounded px-3 py-2 mb-3 text-sm"
              autoFocus
            />
            <textarea
              value={newPlaylistDesc}
              onChange={(e) => setNewPlaylistDesc(e.target.value)}
              placeholder="描述（可选）"
              className="w-full bg-dark-700 rounded px-3 py-2 mb-4 text-sm resize-none"
              rows={2}
            />
            <div className="flex justify-end gap-2">
              <button
                onClick={() => {
                  setShowCreateModal(false)
                  setNewPlaylistName('')
                  setNewPlaylistDesc('')
                }}
                className="px-3 py-1.5 bg-dark-700 hover:bg-dark-600 rounded text-sm transition-colors"
              >
                取消
              </button>
              <button
                onClick={handleCreatePlaylist}
                disabled={!newPlaylistName.trim()}
                className="px-3 py-1.5 bg-primary-600 hover:bg-primary-700 disabled:opacity-50 rounded text-sm transition-colors"
              >
                创建
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 编辑歌单弹窗 */}
      {editingPlaylist && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-dark-800 rounded-lg p-4 w-80">
            <h3 className="font-semibold mb-4">编辑歌单</h3>
            <input
              type="text"
              value={editName}
              onChange={(e) => setEditName(e.target.value)}
              placeholder="歌单名称"
              className="w-full bg-dark-700 rounded px-3 py-2 mb-3 text-sm"
              autoFocus
            />
            <textarea
              value={editDesc}
              onChange={(e) => setEditDesc(e.target.value)}
              placeholder="描述（可选）"
              className="w-full bg-dark-700 rounded px-3 py-2 mb-4 text-sm resize-none"
              rows={2}
            />
            <div className="flex justify-end gap-2">
              <button
                onClick={() => setEditingPlaylist(null)}
                className="px-3 py-1.5 bg-dark-700 hover:bg-dark-600 rounded text-sm transition-colors"
              >
                取消
              </button>
              <button
                onClick={handleUpdatePlaylist}
                disabled={!editName.trim()}
                className="px-3 py-1.5 bg-primary-600 hover:bg-primary-700 disabled:opacity-50 rounded text-sm transition-colors"
              >
                保存
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 点击外部关闭菜单 */}
      {menuOpenId !== null && (
        <div
          className="fixed inset-0 z-0"
          onClick={() => setMenuOpenId(null)}
        />
      )}
    </div>
  )
}

export default PlaylistPanel
