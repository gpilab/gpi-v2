// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;

use gpirs::pyo;
use tauri::api::cli::Matches;

mod from_ui;
mod rust_node;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            parse_cli(app.get_cli_matches().unwrap());
            initialize_python();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            //rust_node::py_add,
            //rust_node::py_add_array,
            from_ui::run_node,
            from_ui::get_python_nodes
        ])
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_fs_watch::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn parse_cli(matches: Matches) {
    if let Some(_help) = matches.args.get("help") {
        println!("help called");
    }
}

fn initialize_python() {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("../nodes/");
    let _ = pyo::initialize_gpipy();
}
