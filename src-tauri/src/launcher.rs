use futures_util::StreamExt;
use serde_json::Value;
use std::io::Cursor;
use tauri::{AppHandle, Manager, Wry};

// --- CONFIGURATION ---
// THAY ĐỔI CÁC GIÁ TRỊ NÀY CHO PHÙ HỢP VỚI REPOSITORY CỦA BẠN
const GITHUB_REPO_OWNER: &str = "NguyenHuynhPhuVinh";
const GITHUB_REPO_NAME: &str = "PvzRHFusionLauncher";
const ZIP_ASSET_NAME: &str = "PC_PVZ-Fusion-3.0.1.zip"; // Tên file zip trong release
const EXECUTABLE_NAME: &str = "PlantsVsZombiesRH.exe"; // Tên file thực thi sau khi giải nén
const GAME_SUBDIR: &str = "game"; // Thư mục con để chứa game trong app data
// ---------------------

#[derive(Clone, serde::Serialize)]
struct DownloadPayload {
    percentage: f64,
    status: String,
}

async fn get_latest_release_url() -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        GITHUB_REPO_OWNER, GITHUB_REPO_NAME
    );

    let response = client
        .get(&url)
        .header("User-Agent", "PvzRhFusionLauncher")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Failed to fetch release info: {}", response.status()));
    }

    let json: Value = response.json().await.map_err(|e| e.to_string())?;

    let assets = json["assets"]
        .as_array()
        .ok_or("No assets found in release")?;

    for asset in assets {
        if let Some(name) = asset["name"].as_str() {
            if name == ZIP_ASSET_NAME {
                return asset["browser_download_url"]
                    .as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| "Asset URL not found".to_string());
            }
        }
    }

    Err(format!("Asset '{}' not found in latest release", ZIP_ASSET_NAME))
}

#[tauri::command]
pub async fn download_and_unzip_game(app_handle: AppHandle<Wry>) -> Result<(), String> {
    let download_url = get_latest_release_url().await?;

    let response = reqwest::get(&download_url)
        .await
        .map_err(|e| e.to_string())?;

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    let mut file_bytes: Vec<u8> = Vec::with_capacity(total_size as usize);

    app_handle.emit("download_progress", DownloadPayload {
        percentage: 0.0,
        status: "Downloading...".to_string(),
    }).unwrap();

    while let Some(item) = stream.next().await {
        let chunk = item.map_err(|e| e.to_string())?;
        file_bytes.extend_from_slice(&chunk);
        downloaded += chunk.len() as u64;
        let percentage = if total_size > 0 {
            (downloaded as f64 / total_size as f64) * 100.0
        } else {
            0.0
        };

        app_handle.emit("download_progress", DownloadPayload {
            percentage,
            status: format!("Downloading... {:.2}%", percentage),
        }).unwrap();
    }

    app_handle.emit("download_progress", DownloadPayload {
        percentage: 100.0,
        status: "Download complete. Unzipping...".to_string(),
    }).unwrap();

    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .ok_or("Could not find app data directory")?;
    let game_dir = app_data_dir.join(GAME_SUBDIR);

    // Xóa thư mục game cũ nếu tồn tại
    if game_dir.exists() {
        std::fs::remove_dir_all(&game_dir).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(&game_dir).map_err(|e| e.to_string())?;

    let cursor = Cursor::new(file_bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let outpath = match file.enclosed_name() {
            Some(path) => game_dir.join(path),
            None => continue,
        };

        if (*file.name()).ends_with('/') {
            std::fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p).map_err(|e| e.to_string())?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath).map_err(|e| e.to_string())?;
            std::io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
        }
    }

     app_handle.emit("download_progress", DownloadPayload {
        percentage: 100.0,
        status: "Installation complete!".to_string(),
    }).unwrap();

    Ok(())
}

#[tauri::command]
pub async fn launch_game(app_handle: AppHandle<Wry>) -> Result<(), String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .ok_or("Could not find app data directory")?;
    
    let game_exe_path = app_data_dir.join(GAME_SUBDIR).join(EXECUTABLE_NAME);

    if !game_exe_path.exists() {
        return Err(format!("Game executable not found at: {:?}", game_exe_path));
    }

    let path_str = game_exe_path.to_str().ok_or("Invalid path")?;

    // Sử dụng tauri-plugin-shell để mở file
    // Lưu ý: Cần định nghĩa scope trong tauri.conf.json
    tauri::Shell::open(
        &app_handle.shell(),
        path_str.to_string(),
        None
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}