use std::thread;
use std::time::Duration;
use crate::threads::save;
use crate::threads::serial::is_usb_tool_connected;
use crate::threads::printer::is_printer_connected;
use crate::threads::camera::{get_camera_status, CameraStatus};
use crate::threads::dialog_test::show_dialog;
use tauri::{AppHandle, Emitter};
use tokio;

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = true;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[setup]{}", msg);
    }
}
        
pub fn spawn_setup_task(app_handle: AppHandle) {
    thread::spawn(move || {
        log("初始化线程已启动");
        
        let app_name = "NanoKVM-Testing";
        match save::init_appdata(app_name) {
            Ok(root_path) => {
                if root_path.exists() {
                    log("应用程序目录初始化成功");
                } else {
                    log("应用程序目录初始化失败");
                    std::process::exit(1);
                }
            }
            Err(e) => {
                log(&format!("初始化失败: {}", e));
                std::process::exit(1);
            }
        }
            // pub board_version: String,
            // pub desktop_mode: String,
            // pub eth_mod: String,
            // pub eth_up_speed: u32,
            // pub eth_down_speed: u32,
            // pub wifi_up_speed: u32,
            // pub wifi_down_speed: u32,
            
        // 延迟2秒后推送初始测试数据，确保前端已经准备好
        std::thread::sleep(std::time::Duration::from_secs(2));
        // 检查机器编号并推送显示
        let machine_number = save::get_config_str("application", "machine_number");
        if  machine_number.is_none() || 
            machine_number.as_ref().map(|n| n == "0").unwrap_or(false) || 
            machine_number.as_ref().map(|n| n.is_empty()).unwrap_or(false) {
            log("机器编号错误，弹窗提示修改，点击确认关闭程序");
            
            show_dialog(app_handle.clone(), format!("检测到机器编号错误，请编辑以下文件[application]中的machine_number：\n\"C:/Users/{}/AppData/Local/NanoKVM-Testing/config/config.toml\"", std::env::var("USERNAME").unwrap()), vec![
                serde_json::json!({ "text": "确定" })
            ], move |result| {
                log(&format!("用户点击了按钮: {}", result));
                std::process::exit(0);
            });
            
            loop {
                // 等待用户点击确定按钮
                thread::sleep(Duration::from_millis(100));
            }
        }
        if let Some(number) = &machine_number {
            log(&format!("机器编号: {}", number));
            if let Err(e) = app_handle.emit("machine-code-update", number) {
                log(&format!("测试任务推送机器编码失败: {}", e));
            }
        }

        // let board_version = save::get_config_str("testing", "board_version");
        // if let Some(version) = &board_version {
        //     log(&format!("板卡版本: {}", version));
        // }
        
        loop{
            // 在普通线程中执行异步函数
            let mut warning_msg = String::new();
            let runtime = tokio::runtime::Runtime::new().unwrap();
            if runtime.block_on(is_usb_tool_connected()) {
                log("USB工具已连接");
            } else {
                log("USB工具未连接");
                warning_msg.push_str("⚠️ USB测试工具未连接，请将USB测试工具连接至本机\n");
            }
            
            // 检查打印机是否连接
            if runtime.block_on(is_printer_connected()) {
                log("打印机已连接");
            } else {
                log("打印机未连接");
                warning_msg.push_str("⚠️ 打印机未连接或打印机驱动未安装，绿灯常亮可能是充电状态，长按侧边按钮开机\n");   
            }

            // 检查摄像头是否连接
            if runtime.block_on(get_camera_status()) != CameraStatus::Disconnected {
                log("摄像头已连接");
            } else {
                log("摄像头未连接");
                warning_msg.push_str("⚠️ HDMI采集卡未连接，或者采集卡连接错误，请连接采集卡的HOST端USB\n");
            }

            if !warning_msg.is_empty() {
                use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
                let ret = Arc::new(AtomicBool::new(false));
                let ret_clone = Arc::clone(&ret);
                
                show_dialog(app_handle.clone(), warning_msg.to_string(), vec![
                    serde_json::json!({ "text": "重新检测" })
                ], move |result| {
                    log(&format!("用户点击了按钮: {}", result));
                    if result == "重新检测" {
                        ret_clone.store(true, Ordering::SeqCst);
                    }
                });
                
                while !ret.load(Ordering::SeqCst) {
                    // 100ms 检查一次
                    thread::sleep(Duration::from_millis(100));
                }

                // 推送关闭弹窗事件
                if let Err(e) = app_handle.emit("hide-dialog", serde_json::json!({})) {
                    log(&format!("弹窗测试任务关闭弹窗失败: {}", e));
                }
                // 等待弹窗关闭动画500ms
                thread::sleep(Duration::from_millis(500));
            } else {
                log("所有测试工具均已连接");
                break;
            }
        }

        loop {
            // 每秒切换一次
            thread::sleep(Duration::from_secs(1));
        }
    });
}