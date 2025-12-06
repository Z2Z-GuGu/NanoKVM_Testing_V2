use std::sync::Mutex;
use tauri::{Emitter, State};

mod test_task;

// 应用状态 - 各个状态作为独立变量
struct AppState {
    machine_code: Mutex<String>,
    server_status: Mutex<String>,
    current_device: Mutex<String>,
    serial_number: Mutex<String>,
    target_ip: Mutex<String>,
    upload_count: Mutex<u32>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            machine_code: Mutex::new(String::new()),
            server_status: Mutex::new(String::new()),
            current_device: Mutex::new(String::new()),
            serial_number: Mutex::new(String::new()),
            target_ip: Mutex::new(String::new()),
            upload_count: Mutex::new(0),
        }
    }
}

// 推送本机编码
#[tauri::command]
fn push_machine_code(code: String, state: State<std::sync::Arc<AppState>>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let mut machine_code = state.machine_code.lock().map_err(|e| e.to_string())?;
    *machine_code = code.clone();
    
    // 直接推送事件
    app_handle.emit("machine-code-update", &code).map_err(|e| e.to_string())?;
    Ok(())
}

// 推送服务器状态
#[tauri::command]
fn push_server_status(status: String, state: State<std::sync::Arc<AppState>>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let mut server_status = state.server_status.lock().map_err(|e| e.to_string())?;
    *server_status = status.clone();
    
    // 直接推送事件
    app_handle.emit("server-status-update", &status).map_err(|e| e.to_string())?;
    Ok(())
}

// 推送待上传数量
#[tauri::command]
fn push_upload_count(count: u32, state: State<std::sync::Arc<AppState>>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let mut upload_count = state.upload_count.lock().map_err(|e| e.to_string())?;
    *upload_count = count;
    
    // 直接推送事件
    app_handle.emit("upload-count-update", count).map_err(|e| e.to_string())?;
    Ok(())
}

// 推送当前硬件
#[tauri::command]
fn push_current_device(device: String, state: State<std::sync::Arc<AppState>>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let mut current_device = state.current_device.lock().map_err(|e| e.to_string())?;
    *current_device = device.clone();
    
    // 直接推送事件
    app_handle.emit("current-device-update", &device).map_err(|e| e.to_string())?;
    Ok(())
}

// 推送当前序列号
#[tauri::command]
fn push_serial_number(serial: String, state: State<std::sync::Arc<AppState>>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let mut serial_number = state.serial_number.lock().map_err(|e| e.to_string())?;
    *serial_number = serial.clone();
    
    // 直接推送事件
    app_handle.emit("serial-number-update", &serial).map_err(|e| e.to_string())?;
    Ok(())
}

// 推送目标IP
#[tauri::command]
fn push_target_ip(ip: String, state: State<std::sync::Arc<AppState>>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let mut target_ip = state.target_ip.lock().map_err(|e| e.to_string())?;
    *target_ip = ip.clone();
    
    // 直接推送事件
    app_handle.emit("target-ip-update", &ip).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::default();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(std::sync::Arc::new(app_state))
        .invoke_handler(tauri::generate_handler![
            push_machine_code,
            push_server_status,
            push_upload_count,
            push_current_device,
            push_serial_number,
            push_target_ip
        ])
        .setup(move |app| {
            let handle = app.handle().clone();
            // 启动测试任务线程 - 定期切换服务器状态
            test_task::spawn_test_task(app.handle().clone());
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
