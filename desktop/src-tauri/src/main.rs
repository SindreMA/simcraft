// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod addon_parser;
mod game_data;
mod models;
mod profileset_generator;
mod result_parser;
mod server;
mod simc_runner;

use std::sync::{mpsc, Mutex};
use tauri::Manager;

struct ApiPort(Mutex<u16>);

#[tauri::command]
fn get_api_port(state: tauri::State<'_, ApiPort>) -> u16 {
    *state.0.lock().unwrap()
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let resource_dir = app
                .path()
                .resource_dir()
                .expect("Failed to resolve resource directory");

            // Load game data
            let data_dir = resource_dir.join("data");
            game_data::load(&data_dir);

            // In production, serve the frontend from the resource dir
            // In dev, the Next.js dev server handles it
            let frontend_dir = if cfg!(debug_assertions) {
                None
            } else {
                let dir = resource_dir.join("frontend");
                if dir.exists() {
                    Some(dir)
                } else {
                    None
                }
            };

            // Start HTTP server
            let resource_dir_clone = resource_dir.clone();
            let (tx, rx) = mpsc::channel();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new()
                    .expect("Failed to create tokio runtime");
                rt.block_on(async {
                    let port = server::start(&resource_dir_clone, frontend_dir).await;
                    tx.send(port).unwrap();
                    tokio::signal::ctrl_c().await.ok();
                });
            });

            let port = rx.recv().expect("Failed to get server port");
            println!("API server running on port {}", port);
            app.manage(ApiPort(Mutex::new(port)));

            Ok(())
        })
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![get_api_port])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
