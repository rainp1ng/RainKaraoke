import { useEffect, useState, useRef } from 'react'
import { FileText } from 'lucide-react'
import { lyricsApi } from '@/lib/api'
import type { Song, LyricsLine } from '@/types'

interface LyricsDisplayProps {
  song: Song | null
  currentTime: number  // 毫秒
}

function LyricsDisplay({ song, currentTime }: LyricsDisplayProps) {
  const [lyrics, setLyrics] = useState<{ format: string; lines: LyricsLine[] } | null>(null)
  const [currentLineIndex, setCurrentLineIndex] = useState(-1)
  const [isLoading, setIsLoading] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)

  // 加载歌词
  useEffect(() => {
    if (!song?.id) {
      setLyrics(null)
      setCurrentLineIndex(-1)
      return
    }

    setIsLoading(true)
    lyricsApi.getLyrics(song.id)
      .then((data) => {
        setLyrics(data)
        setCurrentLineIndex(-1)
      })
      .catch((err) => {
        console.error('Failed to load lyrics:', err)
        setLyrics(null)
      })
      .finally(() => {
        setIsLoading(false)
      })
  }, [song?.id])

  // 更新当前行
  useEffect(() => {
    if (!lyrics || lyrics.lines.length === 0) return

    const timeMs = currentTime * 1000  // 转换为毫秒

    let newIndex = -1
    for (let i = 0; i < lyrics.lines.length; i++) {
      const line = lyrics.lines[i]
      const nextLine = lyrics.lines[i + 1]

      if (timeMs >= line.time) {
        if (!nextLine || timeMs < nextLine.time) {
          newIndex = i
          break
        }
      }
    }

    if (newIndex !== currentLineIndex) {
      setCurrentLineIndex(newIndex)

      // 自动滚动到当前行
      if (containerRef.current && newIndex >= 0) {
        const lineElements = containerRef.current.querySelectorAll('.lyrics-line')
        if (lineElements[newIndex]) {
          lineElements[newIndex].scrollIntoView({
            behavior: 'smooth',
            block: 'center',
          })
        }
      }
    }
  }, [currentTime, lyrics])

  // 渲染逐字高亮
  const renderWords = (line: LyricsLine) => {
    if (!line.words || line.words.length === 0) {
      return line.text
    }

    const timeMs = currentTime * 1000

    return line.words.map((word, idx) => {
      const isActive = timeMs >= word.time && timeMs < word.time + word.duration
      const isPast = timeMs >= word.time + word.duration

      return (
        <span
          key={idx}
          className={`word transition-colors duration-100 ${
            isActive
              ? 'text-primary-400'
              : isPast
              ? 'text-dark-400'
              : 'text-dark-500'
          }`}
        >
          {word.text}
        </span>
      )
    })
  }

  // 无歌曲
  if (!song) {
    return (
      <div className="flex-1 flex items-center justify-center text-dark-400">
        <div className="text-center">
          <FileText className="w-12 h-12 mx-auto mb-2 opacity-50" />
          <p>选择歌曲查看歌词</p>
        </div>
      </div>
    )
  }

  // 加载中
  if (isLoading) {
    return (
      <div className="flex-1 flex items-center justify-center text-dark-400">
        <div className="animate-pulse">加载歌词...</div>
      </div>
    )
  }

  // 无歌词
  if (!lyrics || lyrics.lines.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center text-dark-400">
        <div className="text-center">
          <FileText className="w-12 h-12 mx-auto mb-2 opacity-50" />
          <p>暂无歌词</p>
          <p className="text-sm mt-1">请添加同名 .lrc 或 .ksc 文件</p>
        </div>
      </div>
    )
  }

  return (
    <div
      ref={containerRef}
      className="lyrics-container flex-1 overflow-y-auto p-4 space-y-4"
      style={{
        maskImage: 'linear-gradient(to bottom, transparent 0%, black 15%, black 85%, transparent 100%)',
      }}
    >
      {/* 顶部填充 */}
      <div className="h-24" />

      {lyrics.lines.map((line, index) => {
        const isActive = index === currentLineIndex
        const isPast = index < currentLineIndex

        return (
          <div
            key={index}
            className={`lyrics-line text-center transition-all duration-300 ${
              isActive
                ? 'text-2xl text-white font-medium scale-105'
                : isPast
                ? 'text-lg text-dark-500'
                : 'text-lg text-dark-400'
            }`}
          >
            {renderWords(line)}
          </div>
        )
      })}

      {/* 底部填充 */}
      <div className="h-24" />
    </div>
  )
}

export default LyricsDisplay
