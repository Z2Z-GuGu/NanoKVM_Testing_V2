use std::thread;
use tokio::time::sleep;
use tauri::async_runtime::{spawn};
use std::time::Duration;
use crate::threads::save::{init_appdata, get_config_str, is_app_folder_empty, set_test_status};
use crate::threads::serial::{is_usb_tool_connected};
use crate::threads::printer::is_printer_connected;
use crate::threads::camera::{get_camera_status, CameraStatus};
use crate::threads::dialog_test::{show_dialog, show_dialog_and_wait};
// use crate::threads::test_task::spawn_test_task;
use tauri::{AppHandle, Emitter};
use tokio;
use crate::threads::app::spawn_app_step1_task;
use crate::threads::update_state::{set_upload_count};
use crate::threads::wifi_ap::spawn_wifi_ap;
use crate::threads::static_eth::{set_static_ip_for_testing, STATIC_IP_ENABLE};

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = true;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[setup]{}", msg);
    }
}

// pub fn spawn_step2_file_update(app_handle: AppHandle) {
//     spawn(async move {
//     });
// }

pub fn spawn_setup_task(app_handle: AppHandle) {
    // thread::spawn(move || {
    spawn(async move {
        log("初始化线程已启动");
        let mut ap_ssid = String::new();
        let mut ap_password = String::new();
        let static_ip = "172.168.100.1";
        let target_ip = "172.168.100.2";
        // let static_ip = "192.168.1.7";
        // let target_ip = "192.168.1.15";
        
        // 初始化AppDate
        let app_name = "NanoKVM-Testing";
        match init_appdata(app_name) {
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

        // 延迟2秒后推送初始测试数据，确保前端已经准备好
        // std::thread::sleep(std::time::Duration::from_secs(2));
        sleep(Duration::from_secs(2)).await;

        // 检测配置文件夹
        let mut config_warning_msg = String::new();
        // 检查机器编号并推送显示
        let machine_number = get_config_str("application", "machine_number");
        if  machine_number.is_none() || 
            machine_number.as_ref().map(|n| n.is_empty()).unwrap_or(false) || 
            !machine_number.as_ref().map(|n| n.chars().all(|c| c.is_ascii_digit() && c >= '1' && c <= '9')).unwrap_or(false) {
            log("机器编号错误，弹窗提示修改，点击确认关闭程序");
            config_warning_msg.push_str(&format!("⚠️ 机器编号错误，请编辑以下文件[application]中的machine_number：\n\"C:/Users/{}/AppData/Local/NanoKVM-Testing/config/config.toml\"\n", std::env::var("USERNAME").unwrap()));
        }
        // 检测是否存在APP测试文件
        if is_app_folder_empty() {
            config_warning_msg.push_str(&format!("⚠️ 测试数据文件夹为空，请在下面的位置存放产测软件：\n\"C:/Users/{}/AppData/Local/NanoKVM-Testing/app\"\n", std::env::var("USERNAME").unwrap()));
        }
        
        // 如果有问题就弹窗提示
        if config_warning_msg.is_empty() {
            log("配置文件检查通过");
        } else {
            log(&config_warning_msg);
            show_dialog(app_handle.clone(), format!("{}", config_warning_msg), vec![
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

        // 初始化静态IP
        if STATIC_IP_ENABLE {
            log("初始化静态IP");
            if let Err(e) = set_static_ip_for_testing(static_ip) {
                log(&format!("静态IP配置失败: {}", e));
            }
        }
        
        // 初始化wifi-ap
        if let Some(ap_number) = &machine_number {
            let ssid = format!("NanoKVM_WiFi_Test_{}", ap_number);
            let password = "nanokvmwifi";
            log(&format!("初始化WiFi热点: {} {}", ssid, password));
            let _ = spawn_wifi_ap(&ssid, &password);
            ap_ssid = ssid;
            ap_password = password.to_string();
        }
        // wifi.sh connect_start NanoKVM_WiFi_Test_1 nanokvmwifi

        // 推送机器编号到前端
        if let Some(number) = &machine_number {
            log(&format!("机器编号: {}", number));
            if let Err(e) = app_handle.emit("machine-code-update", number) {
                log(&format!("测试任务推送机器编码失败: {}", e));
            }
        }

        // 推送待上传数量到前端
        set_upload_count(app_handle.clone(), 23);

        // let serial = "Neal0015B";
    
        // // 设置设备信息
        // println!("\n📝 设置设备信息:");
        // let _ = set_test_status(serial, "soc_uid", "6a1760284b50f183");
        // let _ = set_test_status(serial, "soc_uid", "6a1760284b50f183");
        // let _ = set_test_status(serial, "hardware", "Desk-F");
        // let _ = set_test_status(serial, "wifi_exist", "true");

        // 循环检测USB工具、打印机、摄像头是否连接
        loop{
            let mut warning_msg = String::new();
            // 检查USB工具是否连接
            // let runtime = tokio::runtime::Runtime::new().unwrap();
            if is_usb_tool_connected().await {
                log("USB工具已连接");
            } else {
                log("USB工具未连接");
                warning_msg.push_str("⚠️ USB测试工具未连接，请将USB测试工具连接至本机\n");
            }
            
            // 检查打印机是否连接
            if is_printer_connected().await {
                log("打印机已连接");
            } else {
                log("打印机未连接");
                warning_msg.push_str("⚠️ 打印机未连接或打印机驱动未安装，绿灯常亮可能是充电状态，长按侧边按钮开机\n");   
            }

            // 检查摄像头是否连接
            if get_camera_status().await != CameraStatus::Disconnected {
                log("摄像头已连接");
            } else {
                log("摄像头未连接");
                warning_msg.push_str("⚠️ HDMI采集卡未连接，或者采集卡连接错误，请连接采集卡的HOST端USB\n");
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
            let app_step_handle = spawn_app_step1_task(app_handle.clone(), ap_ssid.clone(), ap_password.clone(), static_ip.to_string(), target_ip.to_string());
            app_step_handle.await.unwrap();
        }

        log("测试任务线程已启动，退出初始化线程");
    });
}