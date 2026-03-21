import { useEffect } from 'react'
import { Trash2, GripVertical, ListMusic, ChevronUp } from 'lucide-react'
import { useQueueStore, usePlaybackStore } from '@/stores'
import { formatDuration } from '@/utils/format'

function Queue() {
  const { items, loadQueue, removeFromQueue, moveToNext, clearQueue } = useQueueStore()
  const { play, status, currentSongId } = usePlaybackStore()

  useEffect(() => {
    loadQueue()
  }, [])

  const handlePlaySong = (song: any) => {
    console.log('点击播放歌曲:', song)
    if (song) {
      play(song)
    }
  }

  // 顶歌 - 移到当前播放歌曲之后（下一首位置）
  const handleMoveToTop = async (queueId: number) => {
    if (currentSongId) {
      await moveToNext(queueId, currentSongId)
    }
  }

  // 判断是否可以顶歌（不能顶正在播放的歌曲）
  const canMoveToTop = (item: any, _index: number) => {
    // 必须有正在播放的歌曲，且目标不是正在播放的歌曲
    return currentSongId && item.songId !== currentSongId
  }

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* 标题 */}
      <div className="p-4 border-b border-dark-700 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <ListMusic className="w-5 h-5 text-primary-400" />
          <h2 className="font-semibold">播放队列</h2>
        </div>
        <span className="text-sm text-dark-400">{items.length} 首</span>
      </div>

      {/* 队列列表 */}
      <div className="flex-1 overflow-y-auto">
        {items.length === 0 ? (
          <div className="p-4 text-center text-dark-400">
            <ListMusic className="w-8 h-8 mx-auto mb-2 opacity-50" />
            <p>队列为空</p>
            <p className="text-sm mt-1">从媒体库添加歌曲</p>
          </div>
        ) : (
          <div className="divide-y divide-dark-800">
            {items.map((item, index) => {
              const song = item.song
              const isPlaying = status === 'playing' && currentSongId === item.songId

              return (
                <div
                  key={item.id}
                  className={`p-3 flex items-center gap-2 hover:bg-dark-800/50 transition-colors cursor-pointer ${
                    isPlaying ? 'bg-primary-900/20' : ''
                  }`}
                  onClick={() => song && handlePlaySong(song)}
                >
                  <GripVertical className="w-4 h-4 text-dark-500 cursor-move flex-shrink-0" />
                  <span className="text-dark-400 text-sm w-6 flex-shrink-0">{index + 1}</span>
                  <div className="flex-1 min-w-0">
                    <p className={`font-medium truncate ${isPlaying ? 'text-primary-400' : ''}`}>
                      {song?.title || '未知歌曲'}
                    </p>
                    <p className="text-sm text-dark-400 truncate">
                      {song?.artist || '未知歌手'}
                      {song?.duration && ` · ${formatDuration(song.duration)}`}
                    </p>
                  </div>
                  <div className="flex items-center gap-1 flex-shrink-0">
                    {song?.hasVocal !== undefined && (
                      <div className="flex gap-1 mr-2">
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
                      </div>
                    )}
                    {isPlaying && (
                      <div className="w-2 h-2 bg-primary-500 rounded-full animate-pulse mr-2" />
                    )}
                    {/* 顶歌按钮 - 非正在播放且有歌曲在播放时显示 */}
                    {canMoveToTop(item, index) && (
                      <button
                        onClick={(e) => {
                          e.stopPropagation()
                          handleMoveToTop(item.id)
                        }}
                        className="p-1 hover:bg-dark-600 rounded transition-colors text-dark-400 hover:text-primary-400"
                        title="置顶到下一首"
                      >
                        <ChevronUp className="w-4 h-4" />
                      </button>
                    )}
                    <button
                      onClick={(e) => {
                        e.stopPropagation()
                        removeFromQueue(item.id)
                      }}
                      className="p-1 hover:bg-dark-600 rounded transition-colors text-dark-400 hover:text-red-400"
                      title="移除"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </div>
                </div>
              )
            })}
          </div>
        )}
      </div>

      {/* 清空按钮 */}
      {items.length > 0 && (
        <div className="p-3 border-t border-dark-700 flex gap-2">
          <button
            onClick={clearQueue}
            className="flex-1 py-2 bg-dark-700 hover:bg-dark-600 rounded text-sm transition-colors"
          >
            清空队列
          </button>
        </div>
      )}
    </div>
  )
}

export default Queue
