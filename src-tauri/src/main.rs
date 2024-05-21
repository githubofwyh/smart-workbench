// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::Path;
use std::io::{self, Read};
use serde_json::{json, Value};
use std::process::{Command, Stdio};

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

//                 println!("{}", dir.display());

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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, read_directory, read_package_json_files, exec_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
