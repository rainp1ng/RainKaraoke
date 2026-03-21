/**
 * 音调变换器 - 使用 Web Audio API 实现
 *
 * 通过修改 playbackRate 并补偿速度来近似实现升降调
 * 真正的 pitch shifting 需要 AudioWorklet + FFT 相位声码器
 */

// 半音到频率比例的转换
export function semitonesToRatio(semitones: number): number {
  return Math.pow(2, semitones / 12)
}

// 频率比例到半音的转换
export function ratioToSemitones(ratio: number): number {
  return 12 * Math.log2(ratio)
}

/**
 * 创建一个音频图用于 pitch shifting
 * 注意：这是一个简化的实现，使用 playbackRate 修改 + 时间补偿
 */
export class PitchShiftProcessor {
  private audioContext: AudioContext | null = null
  private sourceNodes: Map<HTMLMediaElement, MediaElementAudioSourceNode> = new Map()
  private pitch: number = 0
  private speed: number = 1.0

  /**
   * 初始化音频上下文
   */
  async init(): Promise<AudioContext> {
    if (!this.audioContext) {
      this.audioContext = new AudioContext()
    }
    if (this.audioContext.state === 'suspended') {
      await this.audioContext.resume()
    }
    return this.audioContext
  }

  /**
   * 将媒体元素连接到音频图
   */
  connectMediaElement(element: HTMLMediaElement): void {
    if (!this.audioContext) return

    // 避免重复连接
    if (this.sourceNodes.has(element)) return

    try {
      const source = this.audioContext.createMediaElementSource(element)
      source.connect(this.audioContext.destination)
      this.sourceNodes.set(element, source)

      // 应用当前的 pitch 设置
      this.applyPitchToElement(element)
    } catch (e) {
      console.error('[PitchShiftProcessor] 连接媒体元素失败:', e)
    }
  }

  /**
   * 断开媒体元素的连接
   */
  disconnectMediaElement(element: HTMLMediaElement): void {
    const source = this.sourceNodes.get(element)
    if (source) {
      source.disconnect()
      this.sourceNodes.delete(element)
    }
  }

  /**
   * 设置音调（半音）
   */
  setPitch(semitones: number): void {
    this.pitch = semitones
    this.applyToAll()
  }

  /**
   * 设置速度
   */
  setSpeed(speed: number): void {
    this.speed = speed
    this.applyToAll()
  }

  /**
   * 获取当前的音调
   */
  getPitch(): number {
    return this.pitch
  }

  /**
   * 应用设置到所有媒体元素
   */
  private applyToAll(): void {
    this.sourceNodes.forEach((_, element) => {
      this.applyPitchToElement(element)
    })
  }

  /**
   * 应用 pitch 设置到单个媒体元素
   */
  private applyPitchToElement(element: HTMLMediaElement): void {
    const pitchRatio = semitonesToRatio(this.pitch)

    // 使用 preservesPitch 属性（现代浏览器支持）
    const el = element as HTMLMediaElement & { preservesPitch?: boolean }

    if ('preservesPitch' in el && el.preservesPitch !== undefined) {
      el.preservesPitch = false
      el.playbackRate = this.speed * pitchRatio
    } else {
      el.playbackRate = this.speed
      console.log('[PitchShiftProcessor] 浏览器不支持 preservesPitch，仅应用速度调整')
    }
  }

  /**
   * 销毁处理器
   */
  destroy(): void {
    this.sourceNodes.forEach((source) => {
      source.disconnect()
    })
    this.sourceNodes.clear()

    if (this.audioContext) {
      this.audioContext.close()
      this.audioContext = null
    }
  }
}

// 全局实例
let pitchProcessor: PitchShiftProcessor | null = null

export function getPitchProcessor(): PitchShiftProcessor {
  if (!pitchProcessor) {
    pitchProcessor = new PitchShiftProcessor()
  }
  return pitchProcessor
}
