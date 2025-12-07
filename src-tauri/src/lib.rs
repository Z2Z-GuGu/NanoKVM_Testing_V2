mod threads;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![])
        .setup(move |app| {
            // 启动串口功能线程
            threads::serial::spawn_serial_task(app.handle().clone());
            // 启动测试任务线程
            threads::test_task::spawn_test_task(app.handle().clone());
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
