import { useEffect, useState, useRef, useCallback } from 'react'
import { effectApi } from '@/lib/api'

interface LevelMeterProps {
  label: string
  getLevel: () => Promise<number>
  isActive?: boolean
}

function LevelMeter({ label, getLevel, isActive = true }: LevelMeterProps) {
  const [level, setLevel] = useState(0)
  const levelRef = useRef<number>(0)

  // 更新频率控制
  const UPDATE_INTERVAL = 33 // ~30fps
  const SMOOTHING_FACTOR = 0.3

  useEffect(() => {
    if (!isActive) return

    let mounted = true

    // 使用 setInterval 进行实际的数据获取
    const intervalId = setInterval(async () => {
      if (!mounted) return
      try {
        const value = await getLevel()
        if (mounted) {
          // 平滑插值
          const smoothed = levelRef.current * (1 - SMOOTHING_FACTOR) + value * SMOOTHING_FACTOR
          levelRef.current = smoothed
          setLevel(smoothed)
        }
      } catch {
        // ignore
      }
    }, UPDATE_INTERVAL)

    return () => {
      mounted = false
      clearInterval(intervalId)
    }
  }, [isActive, getLevel])

  // 计算dB值
  const levelDb = level > 0 ? 20 * Math.log10(level) : -60
  const clampedDb = Math.max(-60, Math.min(0, levelDb))

  // 根据dB值计算百分比 (-60dB = 0%, 0dB = 100%)
  const percentage = ((clampedDb + 60) / 60) * 100

  // 根据电平确定颜色
  const getBarColor = () => {
    if (levelDb > -3) return 'bg-red-500' // 接近削波
    if (levelDb > -6) return 'bg-yellow-500' // 警告区
    return 'bg-green-500' // 正常区
  }

  if (!isActive) return null

  return (
    <div className="bg-dark-800 rounded p-1.5">
      <div className="text-[10px] text-dark-400 mb-1 truncate text-center">{label}</div>
      <div className="h-20 bg-dark-900 rounded relative overflow-hidden">
        {/* 刻度线 */}
        <div className="absolute inset-0 flex flex-col justify-between py-1 pointer-events-none">
          <div className="border-b border-dark-700 h-0" />
          <div className="border-b border-dark-700 h-0" />
          <div className="border-b border-dark-700 h-0" />
          <div className="border-b border-dark-700 h-0" />
        </div>

        {/* 电平条 */}
        <div
          className={`absolute bottom-0 left-0 right-0 ${getBarColor()}`}
          style={{ height: `${percentage}%`, transition: 'height 0.05s linear' }}
        />

        {/* dB 刻度标签 */}
        <div className="absolute right-0.5 top-0 text-[7px] text-dark-500">0</div>
        <div className="absolute right-0.5 top-1/4 text-[7px] text-dark-500">-15</div>
        <div className="absolute right-0.5 top-1/2 text-[7px] text-dark-500">-30</div>
        <div className="absolute right-0.5 top-3/4 text-[7px] text-dark-500">-45</div>
        <div className="absolute right-0.5 bottom-0 text-[7px] text-dark-500">-60</div>
      </div>
      <div className="text-[10px] text-dark-400 mt-1 text-center font-mono">
        {clampedDb.toFixed(1)} dB
      </div>
    </div>
  )
}

// Output Level Meter - 常驻显示
export function OutputLevelMeter() {
  const [isRunning, setIsRunning] = useState(false)

  useEffect(() => {
    const checkState = async () => {
      try {
        const state = await effectApi.getLiveAudioState()
        setIsRunning(state.isRunning)
      } catch {
        setIsRunning(false)
      }
    }

    checkState()
    const interval = setInterval(checkState, 1000)
    return () => clearInterval(interval)
  }, [])

  const getLevel = useCallback(() => effectApi.getOutputLevel(), [])

  return (
    <LevelMeter
      label="输出电平"
      getLevel={getLevel}
      isActive={isRunning}
    />
  )
}

// Chain Level Meter - 只有在效果器链中有levelmeter时才显示
interface ChainLevelMeterProps {
  slotIndex: number
}

export function ChainLevelMeter({ slotIndex }: ChainLevelMeterProps) {
  const [isRunning, setIsRunning] = useState(false)

  useEffect(() => {
    const checkState = async () => {
      try {
        const state = await effectApi.getLiveAudioState()
        setIsRunning(state.isRunning)
      } catch {
        setIsRunning(false)
      }
    }

    checkState()
    const interval = setInterval(checkState, 1000)
    return () => clearInterval(interval)
  }, [])

  const getLevel = useCallback(async () => {
    const value = await effectApi.getLevelMeterValue(slotIndex)
    return value ?? 0
  }, [slotIndex])

  return (
    <LevelMeter
      label={`电平 [${slotIndex + 1}]`}
      getLevel={getLevel}
      isActive={isRunning}
    />
  )
}

export default LevelMeter
