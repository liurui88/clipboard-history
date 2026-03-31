#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::fs;
use tauri::{Emitter, Manager, AppHandle, Listener};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::menu::{Menu, MenuItem};
use chrono::Local;
use base64::{engine::general_purpose::STANDARD, Engine};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_clipboard_manager::ClipboardExt;

#[cfg(target_os = "macos")]
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(target_os = "macos")]
static PREVIOUS_APP_PID: AtomicU64 = AtomicU64::new(0);

fn get_data_dir(app_handle: &AppHandle) -> PathBuf {
    let mut path = app_handle.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap_or_default()
    });
    path.push("clipboard_history");
    let _ = fs::create_dir_all(&path);
    path
}

fn get_history_file_path(app_handle: &AppHandle) -> PathBuf {
    let mut path = get_data_dir(app_handle);
    path.push("history.json");
    path
}

fn save_history_to_file(app_handle: &AppHandle, history: &Vec<ClipboardItem>) {
    let file_path = get_history_file_path(app_handle);
    if let Ok(json) = serde_json::to_string_pretty(history) {
        let _ = fs::write(&file_path, json);
    }
}

fn load_history_from_file(app_handle: &AppHandle) -> Vec<ClipboardItem> {
    let file_path = get_history_file_path(app_handle);
    if file_path.exists() {
        if let Ok(content) = fs::read_to_string(&file_path) {
            if let Ok(history) = serde_json::from_str::<Vec<ClipboardItem>>(&content) {
                return history;
            }
        }
    }
    Vec::new()
}

// macOS native function to read image from clipboard (handles QQ's special format)
#[cfg(target_os = "macos")]
fn read_macos_clipboard_image() -> Option<Vec<u8>> {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};
    
    unsafe {
        let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
        if pasteboard == nil {
            return None;
        }
        
        let types: id = msg_send![pasteboard, types];
        if types == nil {
            return None;
        }
        
        let nsimage_class = class!(NSImage);
        let image: id = msg_send![nsimage_class, alloc];
        let image: id = msg_send![image, initWithPasteboard:pasteboard];
        
        if image != nil {
            let tiff_data: id = msg_send![image, TIFFRepresentation];
            if tiff_data != nil {
                let length: usize = msg_send![tiff_data, length];
                let bytes: *const u8 = msg_send![tiff_data, bytes];
                if !bytes.is_null() && length > 0 {
                    let data = std::slice::from_raw_parts(bytes, length).to_vec();
                    
                    if let Ok(tiff_img) = image::load_from_memory(&data) {
                        let mut png_data = Vec::new();
                        if tiff_img.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png).is_ok() {
                            return Some(png_data);
                        }
                    }
                    return Some(data);
                }
            }
        }
        
        let png_type = std::ffi::CString::new("public.png").unwrap();
        let nsstring_class = class!(NSString);
        let png_type_ns: id = msg_send![nsstring_class, stringWithUTF8String:png_type.as_ptr()];
        
        let data: id = msg_send![pasteboard, dataForType:png_type_ns];
        if data != nil {
            let length: usize = msg_send![data, length];
            let bytes: *const u8 = msg_send![data, bytes];
            if !bytes.is_null() && length > 0 {
                return Some(std::slice::from_raw_parts(bytes, length).to_vec());
            }
        }
        
        None
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ContentType {
    Text,
    Image,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ClipboardItem {
    id: String,
    content_type: ContentType,
    content: String,
    preview: String,
    timestamp: String,
    is_pinned: bool,
}

type SharedHistory = Arc<Mutex<Vec<ClipboardItem>>>;

#[tauri::command]
fn get_clipboard_history(history: tauri::State<SharedHistory>) -> Vec<ClipboardItem> {
    history.lock().unwrap().clone()
}

#[tauri::command]
fn clear_history(app_handle: AppHandle, history: tauri::State<SharedHistory>) {
    let mut h = history.lock().unwrap();
    h.clear();
    let history_vec: Vec<ClipboardItem> = h.clone();
    drop(h);
    save_history_to_file(&app_handle, &history_vec);
}

#[tauri::command]
fn delete_item(app_handle: AppHandle, history: tauri::State<SharedHistory>, id: String) {
    let mut h = history.lock().unwrap();
    h.retain(|item| item.id != id);
    let history_vec: Vec<ClipboardItem> = h.clone();
    drop(h);
    save_history_to_file(&app_handle, &history_vec);
}

#[tauri::command]
fn toggle_pin_item(app_handle: AppHandle, history: tauri::State<SharedHistory>, id: String) {
    let mut h = history.lock().unwrap();
    if let Some(item) = h.iter_mut().find(|item| item.id == id) {
        item.is_pinned = !item.is_pinned;
    }
    let history_vec: Vec<ClipboardItem> = h.clone();
    drop(h);
    save_history_to_file(&app_handle, &history_vec);
}

#[cfg(target_os = "macos")]
fn get_frontmost_app_pid() -> Option<u64> {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let front_app: id = msg_send![workspace, frontmostApplication];
        if front_app != nil {
            let pid: i32 = msg_send![front_app, processIdentifier];
            return Some(pid as u64);
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn activate_app_by_pid(pid: u64) {
    use cocoa::base::id;
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let running_apps: id = msg_send![workspace, runningApplications];
        let count: usize = msg_send![running_apps, count];

        for i in 0..count {
            let app: id = msg_send![running_apps, objectAtIndex:i];
            let app_pid: i32 = msg_send![app, processIdentifier];
            if app_pid as u64 == pid {
                let _: () = msg_send![app, activateWithOptions:1];
                break;
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn is_our_app() -> bool {
    use std::process;
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};

    let our_pid = process::id() as i32;
    
    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let front_app: id = msg_send![workspace, frontmostApplication];
        if front_app != nil {
            let front_pid: i32 = msg_send![front_app, processIdentifier];
            return front_pid == our_pid;
        }
    }
    false
}

#[tauri::command]
fn paste_and_hide(window: tauri::Window) {
    #[cfg(target_os = "macos")]
    {
        let previous_pid = PREVIOUS_APP_PID.load(Ordering::SeqCst);
        let _ = window.hide();
        std::thread::sleep(std::time::Duration::from_millis(200));
        
        if previous_pid != 0 {
            activate_app_by_pid(previous_pid);
            std::thread::sleep(std::time::Duration::from_millis(400));
        } else {
            std::thread::sleep(std::time::Duration::from_millis(400));
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = window.hide();
        std::thread::sleep(std::time::Duration::from_millis(400));
    }

    use enigo::{Enigo, Keyboard, Key, Direction};
    let mut enigo = Enigo::new(&enigo::Settings::default()).unwrap();

    #[cfg(target_os = "macos")]
    {
        let _ = enigo.key(Key::Meta, Direction::Press);
        std::thread::sleep(std::time::Duration::from_millis(50));
        let _ = enigo.key(Key::Unicode('v'), Direction::Click);
        std::thread::sleep(std::time::Duration::from_millis(50));
        let _ = enigo.key(Key::Meta, Direction::Release);
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = enigo.key(Key::Control, Direction::Press);
        std::thread::sleep(std::time::Duration::from_millis(50));
        let _ = enigo.key(Key::Unicode('v'), Direction::Click);
        std::thread::sleep(std::time::Duration::from_millis(50));
        let _ = enigo.key(Key::Control, Direction::Release);
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .invoke_handler(tauri::generate_handler![
            get_clipboard_history,
            clear_history,
            delete_item,
            toggle_pin_item,
            paste_and_hide
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();
            
            // Load history from file
            let loaded_history = load_history_from_file(&app_handle);
            let history: SharedHistory = Arc::new(Mutex::new(loaded_history));
            
            // Save empty history if file doesn't exist (first run)
            {
                let h = history.lock().unwrap();
                if h.is_empty() {
                    drop(h);
                    let mut h = history.lock().unwrap();
                    h.push(ClipboardItem {
                        id: uuid::Uuid::new_v4().to_string(),
                        content_type: ContentType::Text,
                        content: "欢迎使用剪切板历史工具".to_string(),
                        preview: "欢迎使用剪切板历史工具".to_string(),
                        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        is_pinned: false,
                    });
                    save_history_to_file(&app_handle, &*h);
                }
            }

            // Register history as Tauri state
            app.manage(history.clone());

            #[cfg(target_os = "macos")]
            std::thread::spawn(move || {
                let mut last_frontmost_pid: u64 = 0;
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    
                    if let Some(pid) = get_frontmost_app_pid() {
                        if pid != last_frontmost_pid && !is_our_app() {
                            PREVIOUS_APP_PID.store(pid, Ordering::SeqCst);
                            last_frontmost_pid = pid;
                        }
                    }
                }
            });

            let history_for_thread = history.clone();
            let app_handle_for_thread = app_handle.clone();

            std::thread::spawn(move || {
                let mut last_text = String::new();
                let mut last_image_hash: Option<u64> = None;

                loop {
                    std::thread::sleep(std::time::Duration::from_millis(500));

                    let clipboard = app_handle_for_thread.clipboard();

                    match clipboard.read_text() {
                        Ok(text_content) => {
                            if text_content != last_text && !text_content.is_empty() {
                                last_text = text_content.clone();
                                last_image_hash = None;

                                let preview = if text_content.len() > 100 {
                                    format!("{}...", &text_content[..100])
                                } else {
                                    text_content.clone()
                                };

                                let mut h = history_for_thread.lock().unwrap();

                                h.retain(|item| {
                                    if let ContentType::Text = item.content_type {
                                        item.content != text_content
                                    } else {
                                        true
                                    }
                                });

                                h.insert(0, ClipboardItem {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    content_type: ContentType::Text,
                                    content: text_content,
                                    preview,
                                    timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                    is_pinned: false,
                                });

                                if h.len() > 100 {
                                    h.truncate(100);
                                }
                                
                                // Save to file
                                let history_vec: Vec<ClipboardItem> = h.clone();
                                drop(h);
                                save_history_to_file(&app_handle_for_thread, &history_vec);

                                let _ = app_handle_for_thread.emit("clipboard-updated", ());
                            }
                        }
                        _ => {}
                    }

                    let mut image_found = false;
                    let clipboard = app_handle_for_thread.clipboard();
                    match clipboard.read_image() {
                        Ok(image) => {
                            image_found = true;
                            use std::collections::hash_map::DefaultHasher;
                            use std::hash::{Hash, Hasher};

                            let width = image.width();
                            let height = image.height();
                            let rgba_bytes = image.rgba();

                            let png_bytes = if width > 0 && height > 0 {
                                match image::RgbaImage::from_raw(width, height, rgba_bytes.to_vec()) {
                                    Some(rgba_img) => {
                                        let mut png_data = Vec::new();
                                        if let Ok(_) = rgba_img.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png) {
                                            png_data
                                        } else {
                                            rgba_bytes.to_vec()
                                        }
                                    }
                                    None => rgba_bytes.to_vec()
                                }
                            } else {
                                rgba_bytes.to_vec()
                            };

                            let mut hasher = DefaultHasher::new();
                            png_bytes.hash(&mut hasher);
                            let hash = hasher.finish();

                            if last_image_hash != Some(hash) {
                                last_image_hash = Some(hash);
                                last_text.clear();

                                let base64_image = STANDARD.encode(&png_bytes);

                                let mut h = history_for_thread.lock().unwrap();

                                h.insert(0, ClipboardItem {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    content_type: ContentType::Image,
                                    content: base64_image,
                                    preview: "[图片]".to_string(),
                                    timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                    is_pinned: false,
                                });

                                if h.len() > 100 {
                                    h.truncate(100);
                                }

                                // Save to file
                                let history_vec: Vec<ClipboardItem> = h.clone();
                                drop(h);
                                save_history_to_file(&app_handle_for_thread, &history_vec);

                                let _ = app_handle_for_thread.emit("clipboard-updated", ());
                            }
                        }
                        Err(_) => {}
                    }

                    if !image_found {
                        match arboard::Clipboard::new() {
                            Ok(mut cb) => {
                                match cb.get_image() {
                                    Ok(img_data) => {
                                        use std::collections::hash_map::DefaultHasher;
                                        use std::hash::{Hash, Hasher};

                                        let img_bytes = &img_data.bytes;

                                        let mut hasher = DefaultHasher::new();
                                        img_bytes.hash(&mut hasher);
                                        let hash = hasher.finish();

                                        if last_image_hash != Some(hash) {
                                            last_image_hash = Some(hash);
                                            last_text.clear();

                                            let png_bytes = if img_bytes.len() >= 4 {
                                                let is_png = img_bytes[0] == 0x89 && img_bytes[1] == 0x50 && 
                                                            img_bytes[2] == 0x4E && img_bytes[3] == 0x47;
                                                
                                                if is_png {
                                                    img_bytes.to_vec()
                                                } else {
                                                    match image::load_from_memory(img_bytes) {
                                                        Ok(dynamic_img) => {
                                                            let mut png_data = Vec::new();
                                                            if let Ok(_) = dynamic_img.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png) {
                                                                png_data
                                                            } else {
                                                                img_bytes.to_vec()
                                                            }
                                                        }
                                                        Err(_) => img_bytes.to_vec()
                                                    }
                                                }
                                            } else {
                                                img_bytes.to_vec()
                                            };

                                            let base64_image = STANDARD.encode(&png_bytes);

                                            let mut h = history_for_thread.lock().unwrap();

                                            h.insert(0, ClipboardItem {
                                                id: uuid::Uuid::new_v4().to_string(),
                                                content_type: ContentType::Image,
                                                content: base64_image,
                                                preview: "[图片]".to_string(),
                                                timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                                is_pinned: false,
                                            });

                                            if h.len() > 100 {
                                                h.truncate(100);
                                            }

                                            // Save to file
                                            let history_vec: Vec<ClipboardItem> = h.clone();
                                            drop(h);
                                            save_history_to_file(&app_handle_for_thread, &history_vec);

                                            let _ = app_handle_for_thread.emit("clipboard-updated", ());
                                        }
                                    }
                                    Err(_) => {
                                        #[cfg(target_os = "macos")]
                                        {
                                            if let Some(img_data) = read_macos_clipboard_image() {
                                                use std::collections::hash_map::DefaultHasher;
                                                use std::hash::{Hash, Hasher};

                                                let mut hasher = DefaultHasher::new();
                                                img_data.hash(&mut hasher);
                                                let hash = hasher.finish();

                                                if last_image_hash != Some(hash) {
                                                    last_image_hash = Some(hash);
                                                    last_text.clear();

                                                    let base64_image = STANDARD.encode(&img_data);

                                                    let mut h = history_for_thread.lock().unwrap();

                                                    h.insert(0, ClipboardItem {
                                                        id: uuid::Uuid::new_v4().to_string(),
                                                        content_type: ContentType::Image,
                                                        content: base64_image,
                                                        preview: "[图片]".to_string(),
                                                        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                                        is_pinned: false,
                                                    });

                                                    if h.len() > 100 {
                                                        h.truncate(100);
                                                    }

                                                    // Save to file
                                                    let history_vec: Vec<ClipboardItem> = h.clone();
                                                    drop(h);
                                                    save_history_to_file(&app_handle_for_thread, &history_vec);

                                                    let _ = app_handle_for_thread.emit("clipboard-updated", ());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {}
                        }
                    }
                }
            });

            let handler = move |app: &tauri::AppHandle, _shortcut: &tauri_plugin_global_shortcut::Shortcut, event: tauri_plugin_global_shortcut::ShortcutEvent| {
                if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                    if let Some(window) = app.get_webview_window("main") {
                        match window.is_visible() {
                            Ok(visible) => {
                                if visible {
                                    let _ = window.hide();
                                } else {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                            Err(_) => {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                }
            };

            if let Err(_) = app.global_shortcut().on_shortcut("CommandOrControl+Shift+V", handler) {}

            // Setup tray icon and menu
            let quit_i = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "显示", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let tray_app_handle_for_menu = app.handle().clone();
            let tray_app_handle_for_icon = app.handle().clone();
            let _tray = TrayIconBuilder::with_id("tray")
                .menu(&menu)
                .on_menu_event(move |_app, event| {
                    let tray_handle = tray_app_handle_for_menu.clone();
                    if event.id.as_ref() == "quit" {
                        tray_handle.exit(0);
                    } else if event.id.as_ref() == "show" {
                        if let Some(window) = tray_handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .on_tray_icon_event(move |_tray, event| {
                    if let TrayIconEvent::DoubleClick { .. } = event {
                        let tray_handle = tray_app_handle_for_icon.clone();
                        if let Some(window) = tray_handle.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            // Handle macOS app reactivation (when clicking dock icon)
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = event {
                if let Some(window) = _app_handle.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        });
}
