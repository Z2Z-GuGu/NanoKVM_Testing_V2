// 更新前端状态库：使用几个函数直接确定前端状态亮哪个灭哪个
use tauri::{AppHandle, Emitter};
use std::fmt;
use crate::threads::save::{get_config_str};

// 测试项目的状态枚举：'untested' | 'testing' | 'repairing' | 'success' | 'failed' | 'hidden';
#[derive(Debug, PartialEq, Clone)]
pub enum AppTestStatus {
    UnTested    = 0,  // 未测试
    Testing     = 1,  // 测试中
    Repairing   = 2,  // 维修中
    Success     = 3,  // 测试成功
    Failed      = 4,  // 测试失败
    Hidden      = 5,  // 隐藏
}

// 状态枚举
#[derive(Debug, PartialEq, Clone)]
pub enum AppStep1Status {
    Unconnected         = 0,  // 未连接工具
    ConnectedNoKVM      = 1,  // 已连接工具, 未连接KVM
    Uncertain           = 2,  // 状态不确定
    Booted              = 3,  // 已连接KVM，不慎进入BOOT（现在出现AXERA-UBOOT=>，输入boot\n）
    Booting             = 4,  // 已连接KVM，开机中
    BootedLogin         = 5,  // 已连接KVM，已开机（现在出现login）
    LoggedIn            = 6,  // 已连接KVM，已登录（现在出现:~#）
    Download_File       = 7,  // 下载文件中
    Checking_Hardware   = 8,  // 检查硬件中
    Checking_EMMC       = 9,  // 检查eMMC中
    Printing            = 10, // 打印中
    Finished            = 11, // 完成
}

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = true;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[state]{}", msg);
    }
}

impl fmt::Display for AppTestStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            AppTestStatus::UnTested => "untested",
            AppTestStatus::Testing => "testing",
            AppTestStatus::Repairing => "repairing",
            AppTestStatus::Success => "success",
            AppTestStatus::Failed => "failed",
            AppTestStatus::Hidden => "hidden",
        };
        write!(f, "{}", s)
    }
}

// 设置服务器状态
pub fn set_server_state(app_handle: AppHandle, state: bool) {
    if state {
        if let Err(e) = app_handle.clone().emit("server-status-update", "online") {
            log(&format!("设置服务器状态为在线失败: {}", e));
        }
    } else {
        if let Err(e) = app_handle.clone().emit("server-status-update", "offline") {
            log(&format!("设置服务器状态为离线失败: {}", e));
        }
    }
}

// 设置当前硬件类型
pub fn set_current_hardware(app_handle: AppHandle, hardware: &str) {
    let number = get_config_str("testing", "board_version");
    let machine_number = number.unwrap_or_else(|| "A".to_string());
    let current_device = format!("{}-{}", hardware, machine_number);

    if let Err(e) = app_handle.clone().emit("current-device-update", current_device.as_str()) {
        log(&format!("测试任务推送当前设备失败: {}", e));
    }
}

// 设置目标IP
pub fn set_target_ip(app_handle: AppHandle, ip: &str) {
    if let Err(e) = app_handle.clone().emit("target-ip-update", ip) {
        log(&format!("测试任务推送目标IP失败: {}", e));
    }
}

// 设置串号
pub fn set_target_serial(app_handle: AppHandle, serial: &str) {
    if let Err(e) = app_handle.clone().emit("serial-number-update", serial) {
        log(&format!("测试任务推送目标串号失败: {}", e));
    }
}


// 设置测试项目状态(字符串+状态)
pub fn set_step_status(app_handle: AppHandle, test_str: &str, test_status: AppTestStatus) {
    if let Err(e) = app_handle.clone().emit("test-button-status-update", serde_json::json!({
        "buttonId": test_str,
        "status": test_status.to_string()
    })) {
        log(&format!("测试任务推送等待启动按钮状态失败: {}", e));
    }
}

pub fn clean_step1_status(app_handle: AppHandle) {
    set_step_status(app_handle.clone(), "wait_connection", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "wait_boot", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "get_ip", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "detect_hardware", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "download_test", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "emmc_test", AppTestStatus::UnTested);
    set_target_ip(app_handle.clone(), "-");
}

// 从字符串获取已测试项目，并更新状态
pub fn update_tested_steps(app_handle: AppHandle, tested_str: &str) {
    if tested_str.contains("hw_emmc") { set_step_status(app_handle.clone(), "emmc_test", AppTestStatus::Success); }
    if tested_str.contains("hw_emmc") { set_step_status(app_handle.clone(), "emmc_test", AppTestStatus::Success); }
}