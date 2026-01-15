mod threads;

// 从dialog_test模块导入按钮点击处理命令
use crate::threads::dialog_test::handle_button_click;
use std::sync::Arc;
use tauri::State;

// 程序类型枚举
#[derive(Clone, serde::Serialize, serde::Deserialize)]
enum ProgramType {
    Production,
    Post,
}

// 全局程序状态
struct AppState {
    selected_program: std::sync::Mutex<Option<ProgramType>>,
}

// 处理程序选择命令
#[tauri::command]
fn select_program(program: String, state: State<Arc<AppState>>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let mut selected_program = state.inner().selected_program.lock().unwrap();
    
    // 根据选择设置程序类型
    let program_type = match program.as_str() {
        "production" => ProgramType::Production,
        "post" => ProgramType::Post,
        _ => return Err("Invalid program type".to_string()),
    };
    
    *selected_program = Some(program_type.clone());
    
    // 根据选择启动不同的线程
    match program_type {
        ProgramType::Production => {
            println!("启动产测程序所有线程");
            
            // 启动产测程序需要的所有线程
            threads::upload::spawn_upload_task(app_handle.clone());
            threads::setup::spawn_setup_task(app_handle.clone());
            threads::serial::serial_management_task();
            threads::camera::spawn_camera_task();
            
            println!("产测程序: 所有线程已启动");
        },
        ProgramType::Post => {
            println!("启动后测程序线程");
            
            // 后测程序只启动串口线程
            threads::serial::serial_management_task();
            
            println!("后测程序: 串口线程已启动");
        },
    }
    
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化应用状态
    let app_state = Arc::new(AppState {
        selected_program: std::sync::Mutex::new(None),
    });
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![handle_button_click, select_program])
        .setup(move |_app| {
            // 仅初始化全局测试状态，不启动任何线程
            threads::update_state::init_global_state();
            Ok(())
        })
        .manage(app_state)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
