use std::thread;
use tauri::{AppHandle, Emitter};

// 处理前端按钮点击事件的命令
#[tauri::command]
pub fn handle_button_click(button_text: String) {
    println!("前端按钮被按下: {}", button_text);
}

pub fn spawn_dialog_test_task(app_handle: AppHandle) {
    thread::spawn(move || {
        println!("弹窗测试任务线程已启动");
        
        // 延迟3秒后推送测试弹窗，确保前端已经准备好
        std::thread::sleep(std::time::Duration::from_secs(3));

        println!("推送测试弹窗...");
        
        // 推送测试弹窗信息
        if let Err(e) = app_handle.emit("show-dialog", serde_json::json!({
            "message": "这是来自后端的测试弹窗消息\n支持多行文本\n可以显示各种提示信息",
            "buttons": [
                { "text": "确定", "isPrimary": true },
                { "text": "取消" }
            ]
        })) {
            eprintln!("弹窗测试任务推送弹窗失败: {}", e);
        }
        
        // 延迟5秒后推送另一个测试弹窗
        std::thread::sleep(std::time::Duration::from_secs(5));

        println!("推送第二个测试弹窗...");
        
        // 推送第二个测试弹窗信息
        if let Err(e) = app_handle.emit("show-dialog", serde_json::json!({
            "message": "这是另一个测试弹窗，只有一个按钮",
            "buttons": [
                { "text": "OK" }
            ]
        })) {
            eprintln!("弹窗测试任务推送第二个弹窗失败: {}", e);
        }
        
        println!("弹窗测试任务完成");
    });
    
    println!("弹窗测试任务线程创建完成");
}