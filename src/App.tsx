// FILE: src/App.tsx

import { useEffect } from "react";
import { core, event as tauriEvent } from "@tauri-apps/api";
import { exists } from "@tauri-apps/plugin-fs";
import { appDataDir, join } from "@tauri-apps/api/path";

import { Button } from "@/components/ui/button";
import { useLauncherStore } from "@/store/launcherStore";
import "./App.css";

// --- CONFIGURATION ---
const GAME_SUBDIR = "game";
const EXECUTABLE_NAME = "PvzRhFusion.exe";
// ---------------------

function App() {
  const { status, progress, setStatus, setProgress, setIsGameInstalled } =
    useLauncherStore();

  useEffect(() => {
    const checkGameInstallation = async () => {
      try {
        const appDataDirPath = await appDataDir();
        const gameExePath = await join(
          appDataDirPath,
          GAME_SUBDIR,
          EXECUTABLE_NAME
        );
        const isInstalled = await exists(gameExePath);
        setIsGameInstalled(isInstalled);
        setStatus(isInstalled ? "ready" : "not_installed");
      } catch (error) {
        console.error("Failed to check game installation:", error);
        setStatus("not_installed");
      }
    };

    checkGameInstallation();

    const unlisten = tauriEvent.listen<{ percentage: number; status: string }>(
      "download_progress",
      (event) => {
        setProgress(event.payload.percentage);
        if (event.payload.status.includes("complete")) {
          setStatus("ready");
          setIsGameInstalled(true);
        }
      }
    );

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [setIsGameInstalled, setStatus, setProgress]);

  const handleButtonClick = async () => {
    if (status === "not_installed") {
      setStatus("downloading");
      try {
        await core.invoke("download_and_unzip_game");
      } catch (error) {
        console.error("Download failed:", error);
        setStatus("not_installed");
      }
    } else if (status === "ready") {
      try {
        await core.invoke("launch_game");
      } catch (error) {
        console.error("Failed to launch game:", error);
      }
    }
  };

  return (
    <div className="flex min-h-svh flex-col items-center justify-center gap-4 bg-background text-foreground">
      <h1 className="text-3xl font-bold">PvZ RH Fusion Launcher</h1>
      <div className="w-80 text-center">
        {status === "downloading" && `Downloading... ${progress.toFixed(2)}%`}
        {status === "not_installed" && "Game is not installed."}
        {status === "ready" && "Game is ready to play!"}
      </div>
      {status === "downloading" && (
        <div className="w-80 bg-muted rounded-full h-4">
          <div
            className="bg-primary h-4 rounded-full"
            style={{ width: `${progress}%` }}
          ></div>
        </div>
      )}
      <Button onClick={handleButtonClick} disabled={status === "downloading"}>
        {status === "not_installed"
          ? "Install Game"
          : status === "downloading"
          ? "Downloading..."
          : "Play Game"}
      </Button>
    </div>
  );
}

export default App;
