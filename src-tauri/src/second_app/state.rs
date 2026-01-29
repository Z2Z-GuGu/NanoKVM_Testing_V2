// 更新前端状态库：使用几个函数直接确定前端状态亮哪个灭哪个
use tauri::{AppHandle, Emitter};
use std::fmt;
use serde_json;

// use state::Storage;
use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex};

// 状态枚举
#[derive(Debug, PartialEq, Clone)]
pub enum AppStepStatus {
    Unconnected         = 0,  // 未连接工具
    ConnectedNoKVM      = 1,  // 已连接工具, 未连接KVM
    Uncertain           = 2,  // 状态不确定
    Booted              = 3,  // 已连接KVM，不慎进入BOOT（现在出现AXERA-UBOOT=>，输入boot\n）
    Booting             = 4,  // 已连接KVM，开机中
    BootedLogin         = 5,  // 已连接KVM，已开机（现在出现login）
    CheckingState       = 7,  // 检查状态中，已登录（现在出现:~#）
    CheckingHDMI        = 8,  // 检查HDMI中
    CheckingWiFi        = 9,  // 检查WiFi中
    CheckingTouch       = 10, // 检查触摸中
    CheckingKnob        = 11, // 检查旋钮中
    Finished            = 12, // 完成
}

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

pub struct TestState {
    pub wait_connection: AppTestStatus,
    pub wait_power_on: AppTestStatus,
    pub get_status: AppTestStatus,
    pub video_capture: AppTestStatus,
    pub wifi_exist: AppTestStatus,
    pub touch: AppTestStatus,
    pub knob: AppTestStatus,
    pub print_label: AppTestStatus,
    pub error_msg: String,
}

pub static SENCOND_TEST_STATE: OnceCell<Arc<Mutex<TestState>>> = OnceCell::new();

pub fn init_global_state() {
    SENCOND_TEST_STATE.get_or_init(|| {
        Arc::new(Mutex::new(TestState {
            wait_connection: AppTestStatus::UnTested,
            wait_power_on: AppTestStatus::UnTested,
            get_status: AppTestStatus::UnTested,
            video_capture: AppTestStatus::UnTested,
            wifi_exist: AppTestStatus::UnTested,
            touch: AppTestStatus::UnTested,
            knob: AppTestStatus::UnTested,
            print_label: AppTestStatus::UnTested,
            error_msg: String::new(),
        }))
    });
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

// 设置当前硬件类型
pub fn set_current_hardware(app_handle: AppHandle, hardware: &str) {
    if let Err(e) = app_handle.clone().emit("current-device-update", hardware) {
        log(&format!("测试任务推送当前设备失败: {}", e));
    }
}

// 设置串号
pub fn set_target_serial(app_handle: AppHandle, serial: &str) {
    if let Err(e) = app_handle.clone().emit("serial-number-update", serial) {
        log(&format!("测试任务推送目标串号失败: {}", e));
    }
}

pub fn set_state_to_struct(test_str: &str, test_status: AppTestStatus) {
    if let Some(state_arc) = SENCOND_TEST_STATE.get() {
        if let Ok(mut state) = state_arc.lock() {
            match test_str {
                "wait_connection" => { state.wait_connection = test_status; }
                "wait_power_on" => { state.wait_power_on = test_status; }
                "get_status" => { state.get_status = test_status; }
                "video_capture" => { state.video_capture = test_status; }
                "wifi_exist" => { state.wifi_exist = test_status; }
                "touch" => { state.touch = test_status; }
                "knob" => { state.knob = test_status; }
                "print_label" => { state.print_label = test_status; }
                
                _ => { log(&format!("非测试项目: {}", test_str)); }
            }
        }
    }
}

pub fn add_error_msg(msg: &str) {
    if let Some(state_arc) = SENCOND_TEST_STATE.get() {
        if let Ok(mut state) = state_arc.lock() {
            state.error_msg.push_str(msg);
        }
    }
}

fn clear_error_msg() {
    if let Some(state_arc) = SENCOND_TEST_STATE.get() {
        if let Ok(mut state) = state_arc.lock() {
            state.error_msg.clear();
        }
    }
}

pub fn get_error_msg() -> String {
    if let Some(state_arc) = SENCOND_TEST_STATE.get() {
        if let Ok(state) = state_arc.lock() {
            return state.error_msg.clone();
        }
    }
    String::new()
}


// 设置测试项目状态(字符串+状态)
pub fn set_step_status(app_handle: AppHandle, test_str: &str, test_status: AppTestStatus) {
    set_state_to_struct(test_str, test_status.clone());

    if let Err(e) = app_handle.clone().emit("second-test-button-status-update", serde_json::json!({
        "buttonId": test_str,
        "status": test_status.to_string()
    })) {
        log(&format!("测试任务推送等待启动按钮状态失败: {}", e));
    }
}

pub fn clean_step1_status(app_handle: AppHandle) {
    // 左边栏状态
    set_target_serial(app_handle.clone(), "-");
    set_current_hardware(app_handle.clone(), "-");

    set_step_status(app_handle.clone(), "wait_connection", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "wait_power_on", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "get_status", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "video_capture", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "wifi_exist", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "touch", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "knob", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "print_label", AppTestStatus::UnTested);
    
    // 重置error_msg
    clear_error_msg();
}

pub fn all_step_status_is_success() -> bool {
    if let Some(state_arc) = SENCOND_TEST_STATE.get() {
        if let Ok(state) = state_arc.lock() {
            return state.wait_connection == AppTestStatus::Success &&
                   state.wait_power_on == AppTestStatus::Success &&
                   state.get_status == AppTestStatus::Success &&
                   state.video_capture == AppTestStatus::Success &&
                   (state.wifi_exist == AppTestStatus::Success || state.wifi_exist == AppTestStatus::Hidden) &&
                   state.touch == AppTestStatus::Success &&
                   (state.knob == AppTestStatus::Success || state.knob == AppTestStatus::Hidden)
        }
    }
    false
}
