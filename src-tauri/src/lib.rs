mod threads;

// 从dialog_test模块导入按钮点击处理命令
use crate::threads::dialog_test::handle_button_click;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![handle_button_click])
        .setup(move |app| {
            // 启动设置任务线程
            threads::setup::spawn_setup_task(app.handle().clone());
            // 启动串口功能线程
            threads::serial::spawn_serial_task();
            // 启动测试任务线程
            threads::test_task::spawn_test_task(app.handle().clone());
            // 启动摄像头功能线程
            threads::camera::spawn_camera_task();
            // 启动打印机功能线程
            // threads::printer::spawn_printer_task();
            // 启动弹窗测试任务线程
            // threads::dialog_test::spawn_dialog_test_task(app.handle().clone());
            // 启动定时器功能线程
            threads::timer::spawn_timer_task();
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
