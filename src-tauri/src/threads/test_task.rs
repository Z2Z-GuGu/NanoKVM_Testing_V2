use std::thread;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = true;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[test_task]{}", msg);
    }
}

pub fn spawn_test_task(app_handle: AppHandle) {
    thread::spawn(move || {
        log("测试任务线程已启动");
        
        let mut status_toggle = false;

        log("推送初始测试数据...");
        
        // 推送测试数据
        // if let Err(e) = app_handle.emit("machine-code-update", "3") {
        //     log(&format!("测试任务推送机器编码失败: {}", e));
        // }
        if let Err(e) = app_handle.emit("server-status-update", "online") {
            log(&format!("测试任务推送服务器状态失败: {}", e));
        }
        if let Err(e) = app_handle.emit("upload-count-update", 23) {
            log(&format!("测试任务推送上传数量失败: {}", e));
        }
        if let Err(e) = app_handle.emit("current-device-update", "Desk-A") {
            log(&format!("测试任务推送当前设备失败: {}", e));
        }
        if let Err(e) = app_handle.emit("serial-number-update", "Neal0015B") {
            log(&format!("测试任务推送序列号失败: {}", e));
        }
        if let Err(e) = app_handle.emit("target-ip-update", "192.168.1.50") {
            log(&format!("测试任务推送目标IP失败: {}", e));
        }
        
        // 测试按钮状态更新示例
        log("推送测试按钮状态示例...");
        
        // 模拟一些测试按钮状态变化
        if let Err(e) = app_handle.emit("test-button-status-update", serde_json::json!({
            "buttonId": "wait_connection",
            "status": "testing"
        })) {
            log(&format!("测试任务推送等待连接按钮状态失败: {}", e));
        }
        
        std::thread::sleep(std::time::Duration::from_secs(1));
        
        if let Err(e) = app_handle.emit("test-button-status-update", serde_json::json!({
            "buttonId": "wait_boot",
            "status": "success"
        })) {
            log(&format!("测试任务推送等待启动按钮状态失败: {}", e));
        }
        
        std::thread::sleep(std::time::Duration::from_secs(1));
        
        if let Err(e) = app_handle.emit("test-button-status-update", serde_json::json!({
            "buttonId": "uboot",
            "status": "failed"
        })) {
            log(&format!("测试任务推送Uboot按钮状态失败: {}", e));
        }
        
        log("初始测试数据推送完成");
        
        loop {
            // 切换服务器状态
            status_toggle = !status_toggle;
            let status = if status_toggle { "online" } else { "offline" };
            
            // log(&format!("测试任务：推送服务器状态 - {}", status));
            
            // 推送服务器状态更新
            if let Err(e) = app_handle.emit("server-status-update", status) {
                log(&format!("测试任务推送服务器状态失败: {}", e));
            }
            
            // 每秒切换一次
            thread::sleep(Duration::from_secs(1));
        }
    });
    
    log("测试任务线程创建完成");
}