import { create } from 'zustand'
import { getCurrentWindow } from '@tauri-apps/api/window'

interface VideoStore {
  isFullscreen: boolean
  isPiP: boolean
  hasVideo: boolean
  videoRef: HTMLVideoElement | null
  setVideoRef: (ref: HTMLVideoElement | null) => void
  setHasVideo: (hasVideo: boolean) => void
  toggleFullscreen: () => Promise<void>
  togglePiP: () => Promise<void>
  setIsFullscreen: (isFullscreen: boolean) => void
}

export const useVideoStore = create<VideoStore>((set, get) => ({
  isFullscreen: false,
  isPiP: false,
  hasVideo: false,
  videoRef: null,

  setVideoRef: (ref) => set({ videoRef: ref }),

  setHasVideo: (hasVideo) => set({ hasVideo }),

  setIsFullscreen: (isFullscreen) => set({ isFullscreen }),

  toggleFullscreen: async () => {
    try {
      const win = getCurrentWindow()
      const fs = await win.isFullscreen()
      await win.setFullscreen(!fs)
      set({ isFullscreen: !fs })
    } catch (err) {
      console.error('全屏失败:', err)
    }
  },

  togglePiP: async () => {
    const { videoRef, hasVideo } = get()
    if (!videoRef || !hasVideo) return

    try {
      if (document.pictureInPictureElement) {
        await document.exitPictureInPicture()
        set({ isPiP: false })
      } else {
        await videoRef.requestPictureInPicture()
        set({ isPiP: true })
      }
    } catch (err) {
      console.error('画中画失败:', err)
    }
  },
}))
