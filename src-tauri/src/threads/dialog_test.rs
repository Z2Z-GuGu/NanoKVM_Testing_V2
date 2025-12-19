use std::thread;
use std::sync::{Mutex, mpsc};
use lazy_static::lazy_static;
use uuid::Uuid;
use tauri::{AppHandle, Emitter};

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = false;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[dialog_test]{}", msg);
    }
}

// 弹窗结果回调类型
type DialogResultCallback = Box<dyn FnOnce(String) + Send + 'static>;

// 全局存储弹窗回调
lazy_static! {
    static ref DIALOG_CALLBACKS: Mutex<Vec<(String, DialogResultCallback)>> = Mutex::new(Vec::new());
}

// 处理前端按钮点击事件的命令
#[tauri::command]
pub fn handle_button_click(button_text: String, dialog_id: Option<String>) {
    // log(&format!("前端按钮被按下: {}", button_text));
    
    // 如果有dialog_id，尝试找到对应的回调并执行
    if let Some(dialog_id) = dialog_id {
        let mut callbacks = DIALOG_CALLBACKS.lock().unwrap();
        if let Some(index) = callbacks.iter().position(|(id, _)| id == &dialog_id) {
            let (_, callback) = callbacks.remove(index);
            callback(button_text);
        }
    }
}

/// 显示弹窗并等待用户点击结果
/// - `app_handle`: Tauri应用句柄
/// - `message`: 弹窗显示的消息内容
/// - `buttons`: 按钮配置数组
/// - `callback`: 用户点击按钮后的回调函数，参数为按钮文本
pub fn show_dialog<F>(app_handle: AppHandle, message: String, buttons: Vec<serde_json::Value>, callback: F)
where
    F: FnOnce(String) + Send + 'static,
{
    // 生成唯一的dialog_id
    let dialog_id = Uuid::new_v4().to_string();
    
    // 存储回调
    DIALOG_CALLBACKS.lock().unwrap().push((dialog_id.clone(), Box::new(callback)));
    
    // 推送弹窗事件
    if let Err(e) = app_handle.emit("show-dialog", serde_json::json!({
        "message": message,
        "buttons": buttons,
        "dialog_id": dialog_id
    })) {
        log(&format!("推送弹窗失败: {}", e));
        // 清理回调
        let mut callbacks = DIALOG_CALLBACKS.lock().unwrap();
        callbacks.retain(|(id, _)| id != &dialog_id);
    }
}

/// 显示弹窗并等待用户选择，返回用户选择的按钮文本
/// - `app_handle`: Tauri应用句柄
/// - `message`: 弹窗显示的消息内容
/// - `buttons`: 按钮配置数组
/// - 返回值: 用户选择的按钮文本
pub fn show_dialog_and_wait(app_handle: AppHandle, message: String, buttons: Vec<serde_json::Value>) -> String {
    // 创建通道用于接收用户选择结果
    let (tx, rx) = mpsc::channel();
    
    // 生成唯一的dialog_id
    let dialog_id = Uuid::new_v4().to_string();
    
    // 存储回调，当用户点击按钮时通过通道发送结果
    DIALOG_CALLBACKS.lock().unwrap().push((dialog_id.clone(), Box::new(move |result| {
        let _ = tx.send(result);
    })));
    
    // 推送弹窗事件
    if let Err(e) = app_handle.emit("show-dialog", serde_json::json!({
        "message": message,
        "buttons": buttons,
        "dialog_id": dialog_id
    })) {
        log(&format!("推送弹窗失败: {}", e));
        // 清理回调
        let mut callbacks = DIALOG_CALLBACKS.lock().unwrap();
        callbacks.retain(|(id, _)| id != &dialog_id);
        return "取消".to_string(); // 返回默认值
    }
    
    // 阻塞等待用户选择结果
    match rx.recv() {
        Ok(result) => result,
        Err(_) => {
            log("等待用户选择时发生错误");
            "取消".to_string() // 返回默认值
        }
    }
}

pub fn spawn_dialog_test_task(app_handle: AppHandle) {
    thread::spawn(move || {
        log("弹窗测试任务线程已启动");
        
        // 延迟3秒后推送测试弹窗，确保前端已经准备好
        std::thread::sleep(std::time::Duration::from_secs(3));

        log("推送测试弹窗...");
        
        // 推送测试弹窗信息
        show_dialog(app_handle.clone(), "这是来自后端的测试弹窗消息\n支持多行文本\n可以显示各种提示信息".to_string(), vec![
            serde_json::json!({ "text": "确定", "isPrimary": true }),
            serde_json::json!({ "text": "取消" })
        ], |result| {
            log(&format!("用户点击了按钮: {}", result));
        });
        
        // 延迟5秒后关闭第一个弹窗
        std::thread::sleep(std::time::Duration::from_secs(5));
        
        log("关闭第一个测试弹窗...");
        // 推送关闭弹窗事件
        if let Err(e) = app_handle.emit("hide-dialog", serde_json::json!({})) {
            log(&format!("弹窗测试任务关闭弹窗失败: {}", e));
        }
        
        // 等待2秒后推送第二个测试弹窗
        std::thread::sleep(std::time::Duration::from_secs(2));

        log("推送第二个测试弹窗...");
        
        // 推送第二个测试弹窗信息
        show_dialog(app_handle.clone(), "这是另一个测试弹窗，只有一个按钮".to_string(), vec![
            serde_json::json!({ "text": "OK" })
        ], |result| {
            log(&format!("用户点击了按钮: {}", result));
    });
    
    log("弹窗测试任务完成");
});

log("弹窗测试任务线程创建完成");
}