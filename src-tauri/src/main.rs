/*

    Project name: Project Black Pearl
    Date: Thursday, December 16th 2022
    Copyright holder: Project Black Pearl and it's contributors
    Copyright year: 2022

    This software is licensed under the BSD-3-Clause license.
    For more information -> https://opensource.org/licenses/BSD-3-Clause

*/

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use execute::Execute;
use rfd::FileDialog;
use std::thread;
use std::{path::Path, process::Command, vec};
use tauri::CustomMenuItem;
use tauri::SystemTray;
use tauri::SystemTrayMenu;

mod paths;
mod startup;
mod database;

#[tauri::command]
fn handle_scraper(path: String, query: String) {
    // String to path conversion
    let path = Path::new(&path);

    println!(
        "Searching for {} with binary scraper: {}",
        query,
        path.display()
    );

    // Create a command object for the scraper chosen (The command is just it's path)
    // Pass in it's path, a query and the destination folder for the cache file as arguments
    let mut command = Command::new(path);
    command.arg(query);
    command.arg(paths::get_pbp().join("queries"));

    // Run the scraper and tell us about it's exit code
    if let Some(exit_code) = command.execute().unwrap() {
        if exit_code == 0 {
            println!("Scraper query completed successfully.");
        } else {
            println!("Scraper query failed successfully.");
        }
    } else {
        println!("Scraper query interrupted.");
    }
}

#[tauri::command]
fn file_dialog() -> String {
    println!("Executable file dialog opened.");

    // Prompt the user to select a file from their computer as an input
    // For error handling, you can use if- and match statements
    match FileDialog::new()
        .add_filter("Executables", &["exe", "com", "lnk", "cmd", "bat"])
        .set_directory("/")
        .pick_file()
    {
        // If the user picked a file, return the path to the frontend
        Some(file) => file.display().to_string(),
        // If the user just closed the window, without picking a file, return "None" to the frontend
        None => "None".to_string(),
    }
}

#[tauri::command]
fn run_game(path: String) {
    // String to path conversion
    let path = Path::new(&path);

    let mut command = Command::new(path);
    thread::spawn(move || {
        command.execute().expect("Failed to run game");
    });
}

fn main() {
    // Create the usual directories and look for scrapers.
    startup::init();

    // Create the system tray icon
    let tray = SystemTray::new()
        .with_menu(SystemTrayMenu::new().add_item(CustomMenuItem::new("quit", "Quit")));

    // This object is the initial tauri window
    // Tauri commands that can be called from the frontend are to be invoked below
    tauri::Builder::default()
        // Add the system tray to the tauri object and handle it's events
        .system_tray(tray)
        .on_system_tray_event(|_app, event| match event {
            tauri::SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => std::process::exit(0),
                _ => {}
            },
            _ => {}
        })
        // Invoke your commands here
        .invoke_handler(tauri::generate_handler![
            handle_scraper,
            file_dialog,
            run_game,
            database::save_to_db,
            database::get_from_db,
            database::delete_from_db
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        // If you close the window, it won't be terminated, but minimized to your system tray
        .run(|_app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, .. } => {
                api.prevent_exit();
            }
            _ => {}
    });
}
