import { create } from 'zustand'
import { getCurrentWindow } from '@tauri-apps/api/window'

interface VideoStore {
  isFullscreen: boolean
  isPiP: boolean
  hasVideo: boolean
  // 用于触发的标志
  fullscreenTrigger: number
  pipTrigger: number
  setHasVideo: (hasVideo: boolean) => void
  setIsFullscreen: (isFullscreen: boolean) => void
  setIsPiP: (isPiP: boolean) => void
  triggerFullscreen: () => void
  triggerPiP: () => void
}

export const useVideoStore = create<VideoStore>((set, get) => ({
  isFullscreen: false,
  isPiP: false,
  hasVideo: false,
  fullscreenTrigger: 0,
  pipTrigger: 0,

  setHasVideo: (hasVideo) => set({ hasVideo }),

  setIsFullscreen: (isFullscreen) => set({ isFullscreen }),

  setIsPiP: (isPiP) => set({ isPiP }),

  triggerFullscreen: () => {
    set((state) => ({ fullscreenTrigger: state.fullscreenTrigger + 1 }))
  },

  triggerPiP: () => {
    set((state) => ({ pipTrigger: state.pipTrigger + 1 }))
  },
}))
