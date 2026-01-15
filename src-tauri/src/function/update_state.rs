// 更新前端状态库：使用几个函数直接确定前端状态亮哪个灭哪个
use tauri::{AppHandle, Emitter};
use std::fmt;
use serde_json;

// use state::Storage;
use once_cell::sync::OnceCell;
use std::sync::{Arc, Mutex};

pub struct TestState {
    pub wait_connection: AppTestStatus,
    pub wait_boot: AppTestStatus,
    pub get_ip: AppTestStatus,
    pub download_test: AppTestStatus,
    pub detect_hardware: AppTestStatus,
    pub emmc_test: AppTestStatus,
    pub dtb: AppTestStatus,
    pub uboot: AppTestStatus,
    pub kernel: AppTestStatus,
    pub app_install: AppTestStatus,
    pub hdmi_wait_connection: AppTestStatus,
    pub hdmi_io_test: AppTestStatus,
    pub hdmi_loop_test: AppTestStatus,
    pub hdmi_capture_test: AppTestStatus,
    pub hdmi_version: AppTestStatus,
    pub hdmi_write_edid: AppTestStatus,
    pub usb_wait_connection: AppTestStatus,
    pub eth_wait_connection: AppTestStatus,
    pub eth_upload_test: AppTestStatus,
    pub eth_download_test: AppTestStatus,
    pub wifi_wait_connection: AppTestStatus,
    pub wifi_upload_test: AppTestStatus,
    pub wifi_download_test: AppTestStatus,
    pub screen: AppTestStatus,
    pub touch: AppTestStatus,
    pub knob: AppTestStatus,
    pub atx: AppTestStatus,
    pub io: AppTestStatus,
    pub tf_card: AppTestStatus,
    pub uart: AppTestStatus,
    pub auto_start: AppTestStatus,
    pub error_msg: String,
}

pub static CURRENT_TEST_STATE: OnceCell<Arc<Mutex<TestState>>> = OnceCell::new();

pub fn init_global_state() {
    CURRENT_TEST_STATE.get_or_init(|| {
        Arc::new(Mutex::new(TestState {
            wait_connection: AppTestStatus::UnTested,
            wait_boot: AppTestStatus::UnTested,
            get_ip: AppTestStatus::UnTested,
            download_test: AppTestStatus::UnTested,
            detect_hardware: AppTestStatus::UnTested,
            emmc_test: AppTestStatus::UnTested,
            dtb: AppTestStatus::UnTested,
            uboot: AppTestStatus::UnTested,
            kernel: AppTestStatus::UnTested,
            app_install: AppTestStatus::UnTested,
            hdmi_wait_connection: AppTestStatus::UnTested,
            hdmi_io_test: AppTestStatus::UnTested,
            hdmi_loop_test: AppTestStatus::UnTested,
            hdmi_capture_test: AppTestStatus::UnTested,
            hdmi_version: AppTestStatus::UnTested,
            hdmi_write_edid: AppTestStatus::UnTested,
            usb_wait_connection: AppTestStatus::UnTested,
            eth_wait_connection: AppTestStatus::UnTested,
            eth_upload_test: AppTestStatus::UnTested,
            eth_download_test: AppTestStatus::UnTested,
            wifi_wait_connection: AppTestStatus::UnTested,
            wifi_upload_test: AppTestStatus::UnTested,
            wifi_download_test: AppTestStatus::UnTested,
            screen: AppTestStatus::UnTested,
            touch: AppTestStatus::UnTested,
            knob: AppTestStatus::UnTested,
            atx: AppTestStatus::UnTested,
            io: AppTestStatus::UnTested,
            tf_card: AppTestStatus::UnTested,
            uart: AppTestStatus::UnTested,
            auto_start: AppTestStatus::UnTested,
            error_msg: String::new(),
        }))
    });
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

// 状态枚举
#[derive(Debug, PartialEq, Clone)]
pub enum AppStepStatus {
    Unconnected         = 0,  // 未连接工具
    ConnectedNoKVM      = 1,  // 已连接工具, 未连接KVM
    Uncertain           = 2,  // 状态不确定
    Booted              = 3,  // 已连接KVM，不慎进入BOOT（现在出现AXERA-UBOOT=>，输入boot\n）
    Booting             = 4,  // 已连接KVM，开机中
    BootedLogin         = 5,  // 已连接KVM，已开机（现在出现login）
    LoggedIn            = 6,  // 已连接KVM，已登录（现在出现:~#）
    DownloadFile        = 7,  // 下载文件中
    CheckingHardware    = 8,  // 检查硬件中
    CheckingEmmc        = 9,  // 检查eMMC中
    Printing            = 10, // 打印中
    StartStep2          = 11, // 启动Step2
    StartStep3          = 12, // 启动Step3
    Finished            = 13, // 完成
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
    // let number = get_config_str("testing", "board_version");
    // let machine_number = number.unwrap_or_else(|| "A".to_string());
    // let current_device = format!("{}-{}", hardware, machine_number);

    // if let Err(e) = app_handle.clone().emit("current-device-update", current_device.as_str()) {
    if let Err(e) = app_handle.clone().emit("current-device-update", hardware) {
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

// 设置待上传数量
pub fn set_upload_count(app_handle: AppHandle, count: u64) {
    if let Err(e) = app_handle.clone().emit("upload-count-update", count) {
        log(&format!("测试任务推送上传数量失败: {}", e));
    }
}

pub fn set_state_to_struct(test_str: &str, test_status: AppTestStatus) {
    if let Some(state_arc) = CURRENT_TEST_STATE.get() {
        if let Ok(mut state) = state_arc.lock() {
            match test_str {
                "wait_connection" => { state.wait_connection = test_status; }
                "wait_boot" => { state.wait_boot = test_status; }
                "get_ip" => { state.get_ip = test_status; }
                "download_test" => { state.download_test = test_status; }
                "detect_hardware" => { state.detect_hardware = test_status; }
                "emmc_test" => { state.emmc_test = test_status; }
                "dtb" => { state.dtb = test_status; }
                "uboot" => { state.uboot = test_status; }
                "kernel" => { state.kernel = test_status; }
                "app_install" => { state.app_install = test_status; }
                "hdmi_wait_connection" => { state.hdmi_wait_connection = test_status; }
                "hdmi_io_test" => { state.hdmi_io_test = test_status; }
                "hdmi_loop_test" => { state.hdmi_loop_test = test_status; }
                "hdmi_capture_test" => { state.hdmi_capture_test = test_status; }
                "hdmi_version" => { state.hdmi_version = test_status; }
                "hdmi_write_edid" => { state.hdmi_write_edid = test_status; }
                "usb_wait_connection" => { state.usb_wait_connection = test_status; }
                "eth_wait_connection" => { state.eth_wait_connection = test_status; }
                "eth_upload_test" => { state.eth_upload_test = test_status; }
                "eth_download_test" => { state.eth_download_test = test_status; }
                "wifi_wait_connection" => { state.wifi_wait_connection = test_status; }
                "wifi_upload_test" => { state.wifi_upload_test = test_status; }
                "wifi_download_test" => { state.wifi_download_test = test_status; }
                "screen" => { state.screen = test_status; }
                "touch" => { state.touch = test_status; }
                "knob" => { state.knob = test_status; }
                "atx" => { state.atx = test_status; }
                "io" => { state.io = test_status; }
                "tf_card" => { state.tf_card = test_status; }
                "uart" => { state.uart = test_status; }
                "auto_start" => { state.auto_start = test_status; }
                _ => { log(&format!("非测试项目: {}", test_str)); }
            }
        }
    }
}

pub fn add_error_msg(msg: &str) {
    if let Some(state_arc) = CURRENT_TEST_STATE.get() {
        if let Ok(mut state) = state_arc.lock() {
            state.error_msg.push_str(msg);
        }
    }
}

fn clear_error_msg() {
    if let Some(state_arc) = CURRENT_TEST_STATE.get() {
        if let Ok(mut state) = state_arc.lock() {
            state.error_msg.clear();
        }
    }
}

pub fn get_error_msg() -> String {
    if let Some(state_arc) = CURRENT_TEST_STATE.get() {
        if let Ok(state) = state_arc.lock() {
            return state.error_msg.clone();
        }
    }
    String::new()
}


// 设置测试项目状态(字符串+状态)
pub fn set_step_status(app_handle: AppHandle, test_str: &str, test_status: AppTestStatus) {
    set_state_to_struct(test_str, test_status.clone());

    if let Err(e) = app_handle.clone().emit("test-button-status-update", serde_json::json!({
        "buttonId": test_str,
        "status": test_status.to_string()
    })) {
        log(&format!("测试任务推送等待启动按钮状态失败: {}", e));
    }
}

pub fn clean_step1_status(app_handle: AppHandle) {
    // 左边栏状态
    set_target_ip(app_handle.clone(), "-");
    set_target_serial(app_handle.clone(), "-");
    set_current_hardware(app_handle.clone(), "-");

    // step1 测试项目状态
    set_step_status(app_handle.clone(), "wait_connection", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "wait_boot", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "get_ip", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "detect_hardware", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "download_test", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "emmc_test", AppTestStatus::UnTested);
    // step2 测试项目状态
    set_step_status(app_handle.clone(), "dtb", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "uboot", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "kernel", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "app_install", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "hdmi_wait_connection", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "hdmi_io_test", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "hdmi_loop_test", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "hdmi_capture_test", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "hdmi_version", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "hdmi_write_edid", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "usb_wait_connection", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "eth_wait_connection", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "eth_upload_test", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "eth_download_test", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "wifi_wait_connection", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "wifi_upload_test", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "wifi_download_test", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "screen", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "touch", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "knob", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "atx", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "io", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "tf_card", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "uart", AppTestStatus::UnTested);
    // step3 测试项目状态
    set_step_status(app_handle.clone(), "auto_start", AppTestStatus::UnTested);
    set_step_status(app_handle.clone(), "print_error_msg", AppTestStatus::Hidden);
    // 重置error_msg
    clear_error_msg();
}

pub fn all_step_status_is_success() -> bool {
    if let Some(state_arc) = CURRENT_TEST_STATE.get() {
        if let Ok(state) = state_arc.lock() {
            return state.wait_connection == AppTestStatus::Success &&
                   state.wait_boot == AppTestStatus::Success &&
                   state.get_ip == AppTestStatus::Success &&
                   state.download_test == AppTestStatus::Success &&
                   state.detect_hardware == AppTestStatus::Success &&
                   state.emmc_test == AppTestStatus::Success &&
                   state.dtb == AppTestStatus::Success &&
                   state.uboot == AppTestStatus::Success &&
                   state.kernel == AppTestStatus::Success &&
                   state.app_install == AppTestStatus::Success &&
                   state.hdmi_wait_connection == AppTestStatus::Success &&
                   state.hdmi_io_test == AppTestStatus::Success &&
                   state.hdmi_loop_test == AppTestStatus::Success &&
                   state.hdmi_capture_test == AppTestStatus::Success &&
                   state.hdmi_version == AppTestStatus::Success &&
                   state.hdmi_write_edid == AppTestStatus::Success &&
                   state.usb_wait_connection == AppTestStatus::Success &&
                   state.eth_wait_connection == AppTestStatus::Success &&
                   state.eth_upload_test == AppTestStatus::Success &&
                   state.eth_download_test == AppTestStatus::Success &&
                   (state.wifi_wait_connection == AppTestStatus::Success || state.wifi_wait_connection == AppTestStatus::Hidden) &&
                   (state.wifi_upload_test == AppTestStatus::Success || state.wifi_upload_test == AppTestStatus::Hidden) &&
                   (state.wifi_download_test == AppTestStatus::Success || state.wifi_download_test == AppTestStatus::Hidden) &&
                   state.screen == AppTestStatus::Success &&
                   (state.touch == AppTestStatus::Success || state.touch == AppTestStatus::Hidden) &&
                   (state.knob == AppTestStatus::Success || state.knob == AppTestStatus::Hidden) &&
                   state.atx == AppTestStatus::Success &&
                   state.io == AppTestStatus::Success &&
                   (state.tf_card == AppTestStatus::Success || state.tf_card == AppTestStatus::Hidden) &&
                   (state.uart == AppTestStatus::Success || state.uart == AppTestStatus::Hidden);
        }
    }
    false
}
