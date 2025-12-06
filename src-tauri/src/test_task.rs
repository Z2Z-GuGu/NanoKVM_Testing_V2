use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub fn spawn_test_task(app_handle: AppHandle) {
    thread::spawn(move || {
        println!("测试任务线程已启动");
        
        let mut status_toggle = false;
        
        loop {
            // 切换服务器状态
            status_toggle = !status_toggle;
            let status = if status_toggle { "online" } else { "offline" };
            
            println!("测试任务：推送服务器状态 - {}", status);
            
            // 推送服务器状态更新
            if let Err(e) = app_handle.emit("server-status-update", status) {
                eprintln!("测试任务推送服务器状态失败: {}", e);
            }
            
            // 每秒切换一次
            thread::sleep(Duration::from_secs(1));
        }
    });
    
    println!("测试任务线程创建完成");
}