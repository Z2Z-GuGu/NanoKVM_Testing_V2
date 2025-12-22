use std::thread;
use std::time::Duration;
use tauri::async_runtime::spawn;
use tauri::{AppHandle, Emitter};
use crate::threads::serial::{
    is_usb_tool_connected, get_current_data_density, 
    serial_send, detect_serial_string, wait_for_serial_data, execute_command_and_wait};
use crate::threads::dialog_test::{show_dialog_and_wait};
use lazy_static::lazy_static;
use crate::threads::update_state::{AppStep1Status, AppTestStatus, set_step_status, clean_step1_status, set_target_ip, set_current_hardware};

const DATA_DENSITY_THRESHOLD: u64 = 100;            // 数据密度大小判别
const NOT_CONNECTED_KVM_COUNT_THRESHOLD: u64 = 10;  // 未连接KVM超过10次，同步弹窗提示,约10s

// AppStep1Status

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
        let mut app_step1_status = AppStep1Status::Unconnected;
        let mut current_step = app_step1_status.clone();
        let mut not_connected_kvm_count = 0;
        loop {
            // 每轮一定要检测的内容：
            if !is_usb_tool_connected().await {
                app_step1_status = AppStep1Status::Unconnected;
            }
            // 需要以下两个事情：
            // 1. 记录变更，貌似不一定非要记录所有变更，只要是不同状态下进入ConnectedNoKVM就清零计数
            // 2. 如果未连接要再次连接
            // 3. 未知状态其实是暂态，进去一次很快会出来，所以不作为前端的状态变更和后端的时间计数
            if current_step != app_step1_status.clone() {
                if current_step != AppStep1Status::ConnectedNoKVM && 
                   current_step != AppStep1Status::Uncertain &&
                   app_step1_status == AppStep1Status::ConnectedNoKVM {
                    not_connected_kvm_count = 0;
                }
                current_step = app_step1_status.clone();
                log(&format!("应用步骤1状态变更: {:?}", current_step));
            }
            // 检测数据密度
            log(&format!("当前应用步骤1状态: {:?}", app_step1_status));
            match app_step1_status {
                AppStep1Status::Unconnected => {  // 未连接工具
                    log("未连接工具, 进入检测步骤");
                    clean_step1_status(app_handle.clone());
                    if !is_usb_tool_connected().await {
                        log("未连接工具, 弹窗重新检测");
                        let _ = show_dialog_and_wait(app_handle.clone(), "⚠️ USB测试工具未连接，请将USB测试工具连接至本机".to_string(), vec![
                            serde_json::json!({ "text": "确认插入，重新开始" })
                        ]);
                        log("已点击，需要等待弹窗消失后退出");
                        std::thread::sleep(Duration::from_millis(500));
                    } else {
                        log("已连接工具, 转移状态");
                        app_step1_status = AppStep1Status::ConnectedNoKVM;
                        std::thread::sleep(Duration::from_secs(2));
                    }
                    continue;
                }
                AppStep1Status::ConnectedNoKVM => {  // 已连接工具, 未连接KVM
                    clean_step1_status(app_handle.clone());
                    set_step_status(app_handle.clone(), "wait_connection", AppTestStatus::Testing);

                    not_connected_kvm_count += 1;
                    if not_connected_kvm_count >= NOT_CONNECTED_KVM_COUNT_THRESHOLD {
                        // 未连接KVM超过10次，同步弹窗提示
                        set_step_status(app_handle.clone(), "wait_connection", AppTestStatus::Failed);
                        let _ = show_dialog_and_wait(app_handle.clone(), "⚠️ 未检测到KVM，请检查KVM是否连接至测试工具".to_string(), vec![
                            serde_json::json!({ "text": "再次检测" })
                        ]);
                        log("已点击再次检测，需要等待弹窗消失后退出");
                        std::thread::sleep(Duration::from_millis(500));
                        not_connected_kvm_count = 0;
                    }

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
                    let result = detect_serial_string(&patterns, 1000, 0).await;
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
                    set_step_status(app_handle.clone(), "wait_connection", AppTestStatus::Success);
                    set_step_status(app_handle.clone(), "wait_boot", AppTestStatus::Testing);
                    serial_send("boot\n").await;
                    std::thread::sleep(Duration::from_millis(100));
                    app_step1_status = AppStep1Status::ConnectedNoKVM;  // 未连接KVM
                }
                AppStep1Status::Booting => {  // 已连接KVM，开机中
                    log("已连接KVM，开机中, 等待login...");
                    set_step_status(app_handle.clone(), "wait_connection", AppTestStatus::Success);
                    set_step_status(app_handle.clone(), "wait_boot", AppTestStatus::Testing);
                    let patterns = ["login"];
                    let result = detect_serial_string(&patterns, 30000, 10).await;
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
                        "LOW-DENSITY" => {
                            log("等待过程数据密度低于限额, 进入未连接KVM状态");
                            app_step1_status = AppStep1Status::ConnectedNoKVM;  // 未连接KVM
                        }
                        _ => {
                            app_step1_status = AppStep1Status::ConnectedNoKVM;  // 未连接KVM
                        }
                    }
                }
                AppStep1Status::BootedLogin => {  // 已连接KVM，已开机（现在出现login）
                    log("已连接KVM，已开机（现在出现login）, 输入root密码");
                    set_step_status(app_handle.clone(), "wait_connection", AppTestStatus::Success);
                    set_step_status(app_handle.clone(), "wait_boot", AppTestStatus::Testing);
                    if ! execute_command_and_wait("root\n", "Password", 1000).await {
                        app_step1_status = AppStep1Status::ConnectedNoKVM;  // 未连接KVM
                        continue;
                    }
                    log("输入root密码成功, 输入sipeed密码");
                    if ! execute_command_and_wait("sipeed\n", "Welcome", 1000).await {
                        app_step1_status = AppStep1Status::ConnectedNoKVM;  // 未连接KVM
                        continue;
                    }
                    // 等待一部分初始信息
                    std::thread::sleep(Duration::from_millis(100));
                    log("登录成功, 发送回车等待:~#");
                    if ! execute_command_and_wait("\n", ":~#", 1000).await { 
                        app_step1_status = AppStep1Status::ConnectedNoKVM;  // 未连接KVM
                        continue;
                    }
                    app_step1_status = AppStep1Status::LoggedIn;  // 已连接KVM，已登录（现在出现:~#）
                }
                AppStep1Status::LoggedIn => {  // 已连接KVM，已登录（现在出现:~#）
                    log("已连接KVM，已登录（现在出现:~#）");
                    set_step_status(app_handle.clone(), "wait_connection", AppTestStatus::Success);
                    set_step_status(app_handle.clone(), "wait_boot", AppTestStatus::Success);
                    set_step_status(app_handle.clone(), "get_ip", AppTestStatus::Testing);
                    if ! execute_command_and_wait("sudo pkill dhclient\n", ":~#", 1000).await { 
                        log("关闭DHCP超时");
                        // ##
                    }
                    // 清空ip
                    log("清空ip");
                    if ! execute_command_and_wait("sudo ip addr flush dev eth0\n", ":~#", 1000).await {
                        log("清空ip超时");
                        // ##
                    };
                    // 设置静态IP
                    log("设置静态IP");
                    if ! execute_command_and_wait("sudo ip addr add 172.168.100.2/24 dev eth0\n", ":~#", 1000).await { 
                        log("设置静态IP超时");
                        // ##
                    };
                    while ! execute_command_and_wait("ping -c 1 172.168.100.1\n", "1 received", 1000).await {
                        set_step_status(app_handle.clone(), "get_ip", AppTestStatus::Repairing);
                        log("需要发送CTRL C确保退出ping");
                        if ! execute_command_and_wait("\x03", ":~#", 1000).await {
                            log("CTRL+C超时");
                            // ##
                        };
                        // 清空ip
                        log("清空ip");
                        if ! execute_command_and_wait("sudo ip addr flush dev eth0\n", ":~#", 1000).await {
                            log("清空ip超时");
                            // ##
                        };
                        // 设置静态IP
                        log("设置静态IP");
                        if ! execute_command_and_wait("sudo ip addr add 172.168.100.2/24 dev eth0\n", ":~#", 1000).await { 
                            log("设置静态IP超时");
                            // ##
                        };
                    }
                    set_step_status(app_handle.clone(), "get_ip", AppTestStatus::Success);
                    set_target_ip(app_handle.clone(), "172.168.100.2");
                    app_step1_status = AppStep1Status::Checking_Hardware;  // 检查硬件中
                }
                AppStep1Status::Checking_Hardware => {  // 检查硬件中
                    log("检查硬件中");
                    set_step_status(app_handle.clone(), "detect_hardware", AppTestStatus::Testing);

                    // log("检测ATX or Desk");
                    serial_send("i2cdetect -ry 7\n").await;
                    std::thread::sleep(Duration::from_millis(200));
                    serial_send("Y\n").await;
                    let patterns = ["UU", "3c"];
                    let result = detect_serial_string(&patterns, 2000, 10).await;
                    match result.as_str() {
                        "UU" => {
                            log("检测到Desk");
                            set_current_hardware(app_handle.clone(), "Desk");
                        }
                        "3c" => {
                            log("检测到ATX");
                            set_current_hardware(app_handle.clone(), "ATX");
                        }
                        _ => {
                            log("未检测到ATX或Desk, 弹窗判断");
                            let result = show_dialog_and_wait(app_handle.clone(), "⚠️ 可能试屏幕接触不良导致无法判断版本，请手动选择：".to_string(), vec![
                                serde_json::json!({ "text": "ATX" }),
                                serde_json::json!({ "text": "Desk" })
                            ]);
                            if result == "ATX" {
                                set_current_hardware(app_handle.clone(), "ATX");
                            } else {
                                // default Desk
                                set_current_hardware(app_handle.clone(), "Desk");
                            } 
                        }
                    }
                    set_step_status(app_handle.clone(), "detect_hardware", AppTestStatus::Success);
                    // app_step1_status = AppStep1Status::LoggedIn;  // 已连接KVM，已登录（现在出现:~#）

                    // log("检测是否存在wifi模块");
                    if ! execute_command_and_wait("ip a | grep wlan0\n", "state", 1000).await { 
                        log("无wifi");
                        // ##
                    } else {
                        log("有wifi");
                    }

                    // log("检测是否已经产测过+存在串号+6911是否有version");

                    loop {
                        log("sleep");
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
                _ => {}
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    });
}
