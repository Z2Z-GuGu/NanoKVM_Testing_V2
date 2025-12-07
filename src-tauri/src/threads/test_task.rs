use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub fn spawn_test_task(app_handle: AppHandle) {
    thread::spawn(move || {
        println!("测试任务线程已启动");
        
        let mut status_toggle = false;

        // 延迟2秒后推送初始测试数据，确保前端已经准备好
        std::thread::sleep(std::time::Duration::from_secs(2));

        println!("推送初始测试数据...");
        
        // 推送测试数据
        if let Err(e) = app_handle.emit("machine-code-update", "1") {
            eprintln!("测试任务推送机器编码失败: {}", e);
        }
        if let Err(e) = app_handle.emit("server-status-update", "online") {
            eprintln!("测试任务推送服务器状态失败: {}", e);
        }
        if let Err(e) = app_handle.emit("upload-count-update", 23) {
            eprintln!("测试任务推送上传数量失败: {}", e);
        }
        if let Err(e) = app_handle.emit("current-device-update", "Desk-A") {
            eprintln!("测试任务推送当前设备失败: {}", e);
        }
        if let Err(e) = app_handle.emit("serial-number-update", "Neal0015B") {
            eprintln!("测试任务推送序列号失败: {}", e);
        }
        if let Err(e) = app_handle.emit("target-ip-update", "192.168.1.50") {
            eprintln!("测试任务推送目标IP失败: {}", e);
        }
        
        // 测试按钮状态更新示例
        println!("推送测试按钮状态示例...");
        
        // 模拟一些测试按钮状态变化
        if let Err(e) = app_handle.emit("test-button-status-update", serde_json::json!({
            "buttonId": "wait_connection",
            "status": "testing"
        })) {
            eprintln!("测试任务推送等待连接按钮状态失败: {}", e);
        }
        
        std::thread::sleep(std::time::Duration::from_secs(1));
        
        if let Err(e) = app_handle.emit("test-button-status-update", serde_json::json!({
            "buttonId": "wait_boot",
            "status": "success"
        })) {
            eprintln!("测试任务推送等待启动按钮状态失败: {}", e);
        }
        
        std::thread::sleep(std::time::Duration::from_secs(1));
        
        if let Err(e) = app_handle.emit("test-button-status-update", serde_json::json!({
            "buttonId": "uboot",
            "status": "failed"
        })) {
            eprintln!("测试任务推送Uboot按钮状态失败: {}", e);
        }
        
        println!("初始测试数据推送完成");
        
        loop {
            // 切换服务器状态
            status_toggle = !status_toggle;
            let status = if status_toggle { "online" } else { "offline" };
            
            // println!("测试任务：推送服务器状态 - {}", status);
            
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