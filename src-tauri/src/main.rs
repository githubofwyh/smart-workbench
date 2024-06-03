// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs,env};
use std::fs::File;
use std::path::Path;
use std::io::{self, Read};
use std::sync::mpsc::{channel, Receiver};
use serde_json::{json, Value};
use std::process::{Command, Stdio};
use tauri::{SystemTrayMenuItem, SystemTray, CustomMenuItem, SystemTrayMenu, SystemTrayEvent};
use tauri::Manager; // 提供窗口管理器相关的功能
use notify::{RecommendedWatcher, recommended_watcher, Event, EventKind, RecursiveMode, Watcher, Result as NotifyResult, Config};

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// 读取目录
#[tauri::command]
fn read_directory(path: String) -> Result<Vec<String>, String> {
    let entries = fs::read_dir(&path)
        .map_err(|err| err.to_string())?
        .filter_map(|entry| {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_dir() {
                        // If the entry is a directory, return Some(String)
                        Some(path.to_string_lossy().into_owned())
                    } else {
                        // If the entry is not a directory, skip it by returning None
                        None
                    }
                }
                Err(_) => {
                    // If there's an error accessing the entry, skip it by returning None
                    None
                }
            }
        })
        .collect::<Vec<String>>();

    // Return the result as Ok
    Ok(entries)
}

// 读取package.json
#[tauri::command]
fn read_package_json_files(path: String) -> Result<Vec<Value>, String> {
    fn find_package_json_content(dir: &Path, contents: &mut Vec<Value>) -> io::Result<()> {
        if dir.is_dir() {
            // 获取目录下的所有条目
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                // println!("{}", dir.display());

                // 检查是不是文件以及是不是名为 "package.json"
                if path.is_file() && path.file_name() == Some(std::ffi::OsStr::new("package.json")) {
                    // 读取文件并存储内容
                    let mut file_content = String::new();
                    fs::File::open(&path)?.read_to_string(&mut file_content)?;

                    let mut json: Value = serde_json::from_str(&file_content)?;
                    json["path"] = json!(dir);
                    contents.push(json);
                }

                // 如果是一个目录，则尝试在该目录下找到 "package.json"
                if path.is_dir() {
                    let package_json_path = path.join("package.json");
                    if package_json_path.exists() {
                        let mut file_content = String::new();
                        fs::File::open(&package_json_path)?.read_to_string(&mut file_content)?;
                        let mut json: Value = serde_json::from_str(&file_content)?;
                        json["path"] = json!(path);
                        contents.push(json);
                    }
                }
            }
        }
        Ok(())
    }

    let mut package_json_contents = Vec::new();
    let root_dir = Path::new(&path);

    // 开始迭代根目录下的项目并收集内容
    if let Err(error) = find_package_json_content(&root_dir, &mut package_json_contents) {
        return Err(error.to_string());
    }

    // 返回所有找到的 package.json 文件内容
    if !package_json_contents.is_empty() {
        Ok(package_json_contents)
    } else {
        Err("No package.json files found in the specified path.".to_string())
    }
}

// 执行脚本命令
#[tauri::command]
fn exec_command(path: String, command: String) -> Result<String, String> {
    // 在给定的路径下执行命令
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(Path::new(&path)) // 设置命令执行时的当前目录
        .stdout(Stdio::piped()) // 捕获标准输出
        .stderr(Stdio::piped()) // 捕获标准错误
        .output()
        .map_err(|e| e.to_string())?; // 如果执行出错，转换错误为字符串返回

    // 检查执行是否成功
    if output.status.success() {
        // 如果命令执行成功，返回标准输出的内容
        String::from_utf8(output.stdout).map_err(|e| e.to_string())
    } else {
        // 如果命令执行失败，返回标准错误的内容
        String::from_utf8(output.stderr).map_err(|e| e.to_string())
    }
}

// 读取指定的临时文件
#[tauri::command]
fn read_tmp_file() -> Result<String, String> {
    // 文件路径
    let file_path = "/tmp/devtools.config";

    // 以只读方式打开文件
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => return Err(e.to_string()),
    };

    // 读取文件内容
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => Ok(contents),
        Err(e) => Err(e.to_string()),
    }
}

// 启用代理的方法
#[tauri::command]
fn enable_proxy(host: &str, port: &str) {
    let command = format!(
        "networksetup -setwebproxy 'Wi-Fi' {} {} && networksetup -setsecurewebproxy 'Wi-Fi' {} {}",
        host, port, host, port
    );

    if let Ok(_) = Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
    {
        println!("Proxy enabled successfully");
    } else {
        println!("Failed to enable proxy");
    }
}

// 禁用代理的方法
#[tauri::command]
fn disable_proxy() {
    let command = "networksetup -setwebproxystate 'Wi-Fi' off && networksetup -setsecurewebproxystate 'Wi-Fi' off";

    if let Ok(_) = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
    {
        println!("Proxy disabled successfully");
    } else {
        println!("Failed to disable proxy");
    }
}


fn main() {
    let host = "localhost"; // Example host
    let port = "8080"; // Example port

    // 创建菜单项
    let quit = CustomMenuItem::new("quit".to_string(), "退出");
    let enable_proxy_item = CustomMenuItem::new("enable_proxy_item".to_string(), "一键开启代理");
    let disable_proxy_item = CustomMenuItem::new("disable_proxy_item".to_string(), "一键关闭代理");

    // 创建系统托盘菜单
    let tray_menu = SystemTrayMenu::new().add_item(enable_proxy_item).add_item(disable_proxy_item).add_native_item(SystemTrayMenuItem::Separator).add_item(quit);

    // 创建系统托盘实例
    let tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .system_tray(tray)
        .setup(|app| {
                // 启动文件监控
                // start_file_watcher(app.handle());

                // 默认开启代理
                enable_proxy(host, port);

                let enable_proxy_item = app.tray_handle().get_item("enable_proxy_item");
                let disable_proxy_item = app.tray_handle().get_item("disable_proxy_item");

                    // 一旦启动，立即检查代理状态
//                     check_proxy_status(enable_proxy_item, disable_proxy_item);
//
//                     tauri::async_runtime::spawn(async move {
//                         loop {
//                             // 每隔一定时间检查代理状态
//                             tauri::async_runtime::sleep(std::time::Duration::from_secs(60)).await; // 检查间隔
//                             check_proxy_status(proxy_enabled_item, proxy_disabled_item);
//                         }
//                     });

                    Ok(())
                })
        .on_system_tray_event(|_app, event| match event {
                    SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                        "quit" => {
                            std::process::exit(0);
                        }
                        "enable_proxy_item" => {
                            // 执行启用代理的命令
                            enable_proxy(host, port);
                        }
                        "disable_proxy_item" => {
                            // 执行关闭代理的命令
                            disable_proxy();
                        }
                        _ => {}
                    }
                    _ => {}
                })
        .invoke_handler(tauri::generate_handler![greet, read_directory, read_package_json_files, exec_command, read_tmp_file, enable_proxy, disable_proxy])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// fn start_file_watcher(handle: tauri::AppHandle) {
//     let path = Path::new("/tmp/tempfile.devtools");
//
//     // Create a channel to receive the events.
//     let (tx, rx) = channel();
//
//     // RecommendedWatcher is an alias for a platform-specific watcher implementation.
//     let mut watcher = RecommendedWatcher::new(move |res: NotifyResult<Event>| {
//         if tx.send(res).is_err() {
//             // Log detailed error if the send fails
//             eprintln!("Failed to send event through channel");
//         }
//     }, Config::default()).expect("Failed to create watcher");
//
//     // Ensuring the watcher is not prematurely dropped.
// //     let watcher = Box::new(watcher);
//
//     // Add a path to be watched. All files and directories at that path and below will be monitored for changes.
//     watcher.watch(path, RecursiveMode::NonRecursive).expect("Failed to watch path");
//
//     // First access, directly read/write the file
//     if let Ok(contents) = fs::read_to_string(path) {
//         println!("Initial file content: {}", contents);
//     } else {
//         if fs::write(path, "Initial content").is_err() {
//             // Log detailed error if initial write fails
//             eprintln!("Failed to write initial content to the file: {:?}", path);
//         }
//     }
//
//     // Spawn a thread to monitor the changes, passing the watcher to ensure it is not dropped
//     std::thread::spawn(move || {
//         handle_events(handle, rx);
//     });
//
//     // Keep the watcher in the main scope
//     std::thread::spawn(move || {
//         loop {
//             std::thread::sleep(std::time::Duration::from_secs(60));
//         }
//     }).join().unwrap();
// }
//
// fn handle_events(handle: tauri::AppHandle, rx: Receiver<NotifyResult<Event>>) {
//     // Process events from the watcher
//     loop {
//         match rx.recv() {
//             Ok(Ok(event)) => {
//                 if matches!(event.kind, EventKind::Modify(_)) {
//                     // Notify the frontend about the file update
//                     if handle.emit_all("file-updated", "File has been updated").is_err() {
//                         // Log detailed error if the event emission fails
//                         eprintln!("Failed to emit event to frontend");
//                     }
//                 }
//             }
//             Ok(Err(e)) => eprintln!("Watch error: {:?}", e),
//             Err(RecvError) => {
//                 eprintln!("Channel receive error: RecvError");
//                 break; // Exiting the loop if the channel is disconnected
//             },
//         }
//     }
// }
