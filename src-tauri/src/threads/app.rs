use std::thread;
use std::time::Duration;
use tauri::async_runtime::spawn;
use tauri::{AppHandle, Emitter};
use crate::threads::serial::{
    is_usb_tool_connected, get_current_data_density, 
    serial_send, detect_serial_string, wait_for_serial_data, execute_command_and_wait};
use crate::threads::dialog_test::{show_dialog_and_wait};
use lazy_static::lazy_static;

const DATA_DENSITY_THRESHOLD: u64 = 100;          // 数据密度大小判别

/*
    应用步骤1状态全局变量
    APP_STEP1_STATUS = 0  // 未连接工具
    APP_STEP1_STATUS = 1  // 已连接工具, 未连接KVM
    APP_STEP1_STATUS = 2  // 状态不确定
    APP_STEP1_STATUS = 3  // 已连接KVM，不慎进入BOOT（现在出现AXERA-UBOOT=>，输入boot\n）
    APP_STEP1_STATUS = 4  // 已连接KVM，开机中
    APP_STEP1_STATUS = 5  // 已连接KVM，已开机（现在出现login）
    APP_STEP1_STATUS = 6  // 已连接KVM，已登录（现在出现:~#）
*/
// 状态枚举
#[derive(Debug, PartialEq)]
pub enum AppStep1Status {
    Unconnected     = 0,  // 未连接工具
    ConnectedNoKVM  = 1,  // 已连接工具, 未连接KVM
    Uncertain       = 2,  // 状态不确定
    Booted          = 3,  // 已连接KVM，不慎进入BOOT（现在出现AXERA-UBOOT=>，输入boot\n）
    Booting         = 4,  // 已连接KVM，开机中
    BootedLogin     = 5,  // 已连接KVM，已开机（现在出现login）
    LoggedIn        = 6,  // 已连接KVM，已登录（现在出现:~#）
}

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = true;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[app]{}", msg);
    }
}

// lazy_static! {
//     pub static ref APP_STEP1_STATUS: Mutex<AppStep1Status> = Mutex::new(AppStep1Status::Unconnected);    // 应用步骤1状态全局变量
// }

pub fn spawn_app_step1_task(app_handle: AppHandle) {
    spawn(async move {
        let mut app_step1_status = AppStep1Status::ConnectedNoKVM;
        let mut current_usb_tool_connected = true;
        loop {
            // 检测工具是否连接
            log(&format!("当前USB工具连接状态: {:?}", current_usb_tool_connected));
            if current_usb_tool_connected != is_usb_tool_connected().await {
                log("状态不等");
                current_usb_tool_connected = !current_usb_tool_connected;
                if current_usb_tool_connected {
                    log("已连接工具, 不确认是否连接KVM");
                    app_step1_status = AppStep1Status::ConnectedNoKVM;  // 已连接工具, 未连接KVM
                    std::thread::sleep(Duration::from_secs(1));
                } else {
                    log("未连接工具, 弹窗重新检测");
                    app_step1_status = AppStep1Status::Unconnected;  // 未连接工具
                    let _ = show_dialog_and_wait(app_handle.clone(), "⚠️ USB测试工具未连接，请将USB测试工具连接至本机".to_string(), vec![
                        serde_json::json!({ "text": "确认插入，重新开始" })
                    ]);
                    current_usb_tool_connected = true;
                    continue;
                }
            }

            // 检测数据密度
            log(&format!("当前应用步骤1状态: {:?}", app_step1_status));
            match app_step1_status {
                AppStep1Status::Unconnected => {  // 未连接工具
                    log("未连接工具, 等待连接");
                    continue;
                }
                AppStep1Status::ConnectedNoKVM => {  // 已连接工具, 未连接KVM
                    log("已连接工具, 未连接KVM, 检测数据密度");
                    let current_data_density = get_current_data_density().await;
                    log(&format!("当前数据密度: {:?}", current_data_density));
                    if current_data_density == 0 {
                        app_step1_status = AppStep1Status::Uncertain;           // 进入不确定状态
                    } else {
                        app_step1_status = AppStep1Status::Booting;             // 进入开机中状态
                    }
                    continue;
                }
                AppStep1Status::Uncertain => {  // 状态不确定
                    log("状态不确定, 发送换行符");
                    serial_send("\n").await;
                    std::thread::sleep(Duration::from_millis(100));
                    let patterns = ["login", ":~#", "AXERA-UBOOT=>"];
                    let result = detect_serial_string(&patterns, 1000).await;
                    log(&format!("检测结果: {}", result));
                    match result.as_str() {
                        "login" => {
                            log("检测到login, 进入开机中状态");
                            app_step1_status = AppStep1Status::BootedLogin;  // 已连接KVM，已开机（现在出现login）
                        }
                        ":~#" => {
                            log("检测到:~#, 进入已登录状态");
                            app_step1_status = AppStep1Status::LoggedIn;  // 已连接KVM，已登录（现在出现:~#）
                        }
                        "AXERA-UBOOT=>" => {
                            log("检测到AXERA-UBOOT=>, 进入BOOT状态");
                            app_step1_status = AppStep1Status::Booted;  // 已连接KVM，不慎进入BOOT（现在出现AXERA-UBOOT=>，输入boot\n）
                        }
                        "UNMATCHED" => {
                            log("检测到UNMATCHED, 进入开机中状态");
                            app_step1_status = AppStep1Status::Booting;  // 已连接KVM，开机中
                        }
                        "NO-DATA" => {
                            log("检测到NO-DATA, 进入未连接KVM状态");
                            app_step1_status = AppStep1Status::ConnectedNoKVM;  // 未连接KVM
                        }
                        _ => {
                            log(&format!("其他情况: {}", result));
                            app_step1_status = AppStep1Status::ConnectedNoKVM;  // 未连接KVM
                        }
                    }
                }
                AppStep1Status::Booted => {  // 已连接KVM，不慎进入BOOT（现在出现AXERA-UBOOT=>，输入boot\n）
                    log("已连接KVM，不慎进入BOOT（现在出现AXERA-UBOOT=>，输入boot\n）, 发送boot\n");
                    serial_send("boot\n").await;
                    std::thread::sleep(Duration::from_millis(100));
                    app_step1_status = AppStep1Status::ConnectedNoKVM;  // 未连接KVM
                }
                AppStep1Status::Booting => {  // 已连接KVM，开机中
                    log("已连接KVM，开机中, 等待login...");
                    let patterns = ["login"];
                    let result = detect_serial_string(&patterns, 30000).await;
                    match result.as_str() {
                        "login" => {
                            app_step1_status = AppStep1Status::BootedLogin;  // 已连接KVM，已开机（现在出现login）
                        }
                        "UNMATCHED" => {
                            // 开着，但是超时了，也有可能关不上了，建议是拔掉再试一次，或者打印贴纸
                            // ##
                        }
                        "NO-DATA" => {
                            app_step1_status = AppStep1Status::ConnectedNoKVM;  // 未连接KVM
                        }
                        _ => {
                            app_step1_status = AppStep1Status::ConnectedNoKVM;  // 未连接KVM
                        }
                    }
                }
                AppStep1Status::BootedLogin => {  // 已连接KVM，已开机（现在出现login）
                    log("已连接KVM，已开机（现在出现login）, 输入root密码");
                    if !execute_command_and_wait("root", "Password:", 1000).await {
                        // 登录超时，建议是拔掉再试一次，或者打印贴纸
                        // ##
                    }
                    if !execute_command_and_wait("sipeed", ":~#", 2000).await {
                        // 输入密码超时，建议是拔掉再试一次，或者打印贴纸
                        // ##
                    }
                    app_step1_status = AppStep1Status::LoggedIn;  // 已连接KVM，已登录（现在出现:~#）
                }
                AppStep1Status::LoggedIn => {  // 已连接KVM，已登录（现在出现:~#）
                    log("已连接KVM，已登录（现在出现:~#）");
                    if ! execute_command_and_wait("sudo pkill dhclient \r\n", "#", 1000).await { 
                        // 关闭DHCP超时，建议是拔掉再试一次，或者打印贴纸
                        // ##
                    }
                    while ! execute_command_and_wait("ping -c 1 172.168.100.1\r\n", "time", 1000).await {
                        // 清空ip
                        log("清空ip");
                        if ! execute_command_and_wait("sudo ip addr flush dev eth0\r\n", "#", 1000).await { 
                            // 清空ip超时，建议是拔掉再试一次，或者打印贴纸
                            // ##
                        };
                        // 设置静态IP
                        log("设置静态IP");
                        if ! execute_command_and_wait("sudo ip addr add 172.168.100.2/24 dev eth0\r\n", "#", 1000).await { 
                            // 设置静态IP超时，建议是拔掉再试一次，或者打印贴纸
                            // ##
                        };
                    }
                    loop {
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
                _ => {}
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    });
}
