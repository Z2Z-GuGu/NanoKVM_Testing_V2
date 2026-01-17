use std::thread;
use tokio::time::sleep;
use tauri::async_runtime::{spawn};
use std::time::Duration;
use crate::function::serial::{is_usb_tool_connected};
use crate::function::printer::is_printer_connected;
use crate::function::dialog_test::{show_dialog_and_wait};
// use crate::threads::test_task::spawn_test_task;
use tauri::{AppHandle};
use tokio;
use crate::second_app::app::spawn_app_step1_task;

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = true;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[setup]{}", msg);
    }
}

pub fn spawn_setup_task(app_handle: AppHandle) {
    // thread::spawn(move || {
    spawn(async move {
        log("初始化线程已启动");

        // 延迟2秒后推送初始测试数据，确保前端已经准备好
        sleep(Duration::from_secs(2)).await;

        // 循环检测USB工具、打印机、摄像头是否连接
        loop{
            let mut warning_msg = String::new();
            // 检查USB工具是否连接
            // let runtime = tokio::runtime::Runtime::new().unwrap();
            if is_usb_tool_connected().await {
                log("USB工具已连接");
            } else {
                log("USB工具未连接");
                warning_msg.push_str("⚠️ USB测试工具未连接或正在占用，请将USB测试工具连接至本机或关闭占用软件\n");
            }
            
            // 检查打印机是否连接
            if is_printer_connected().await {
                log("打印机已连接");
            } else {
                log("打印机未连接");
                warning_msg.push_str("⚠️ 打印机未连接或打印机驱动未安装，绿灯常亮可能是充电状态，长按侧边按钮开机\n");   
            }

            if !warning_msg.is_empty() {
                let ret = show_dialog_and_wait(app_handle.clone(), warning_msg.to_string(), vec![
                    serde_json::json!({ "text": "重新检测" })
                ]);
                if ret == "重新检测" {
                    // 等待弹窗关闭动画500ms
                    thread::sleep(Duration::from_millis(500));
                    continue;
                }
            } else {
                log("所有测试工具均已连接");
                break;
            }
        }
        // serial_data_management_task(app_handle.clone());
        loop {
            let app_step_handle = spawn_app_step1_task(app_handle.clone());
            app_step_handle.await.unwrap();
        }
    });
}