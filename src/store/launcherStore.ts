import { create } from "zustand";

type LauncherStatus = "checking" | "not_installed" | "downloading" | "ready";

interface LauncherState {
  status: LauncherStatus;
  progress: number;
  isGameInstalled: boolean;
  setStatus: (status: LauncherStatus) => void;
  setProgress: (progress: number) => void;
  setIsGameInstalled: (isInstalled: boolean) => void;
}

export const useLauncherStore = create<LauncherState>((set) => ({
  status: "checking",
  progress: 0,
  isGameInstalled: false,
  setStatus: (status) => set({ status }),
  setProgress: (progress) => set({ progress }),
  setIsGameInstalled: (isInstalled) => set({ isGameInstalled: isInstalled }),
}));
