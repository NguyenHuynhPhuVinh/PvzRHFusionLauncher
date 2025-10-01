// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod launcher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            launcher::download_and_unzip_game,
            launcher::launch_game
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
