use std::thread;
use std::time::Duration;
use tauri::async_runtime::spawn;
use tauri::{AppHandle, Emitter};
use crate::threads::serial::{
    is_usb_tool_connected, get_current_data_density, 
    serial_send, detect_serial_string, wait_for_serial_data, execute_command_and_wait};
use crate::threads::dialog_test::{show_dialog_and_wait};
use lazy_static::lazy_static;
use crate::threads::update_state::{AppStep1Status, AppTestStatus, 
    set_step_status, clean_step1_status, set_target_ip, set_current_hardware, set_target_serial};
use tauri::async_runtime::JoinHandle;
use crate::threads::server::spawn_file_server_task;
use crate::threads::ssh::ssh_execute_command;
use crate::threads::save::{get_config_str};
use crate::threads::printer::{is_printer_connected, generate_image_with_params, print_image, PRINTER_ENABLE, TARGET_PRINTER};

const DATA_DENSITY_THRESHOLD: u64 = 100;            // 数据密度大小判别
const NOT_CONNECTED_KVM_COUNT_THRESHOLD: u64 = 10;  // 未连接KVM超过10次，同步弹窗提示,约10s

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = true;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[step2]{}", msg);
    }
}

pub fn spawn_step2_file_update(app_handle: AppHandle) {
    spawn(async move {
        log("更新KVM文件");
        set_step_status(app_handle.clone(), "dtb", AppTestStatus::Testing);

        match ssh_execute_command("/root/NanoKVM_Pro_Testing/test_sh/04_update_file.sh dtb").await {
            Ok(output) => {
                if output.contains("dtb文件更新成功") {
                    log("dtb文件更新成功");
                    set_step_status(app_handle.clone(), "dtb", AppTestStatus::Success);
                } else {
                    log("dtb文件更新失败");
                    set_step_status(app_handle.clone(), "dtb", AppTestStatus::Failed);
                }
            }
            Err(e) => {
                log(&format!("SSH命令执行失败: {}", e));
            }
        }
    });
}

// pub fn spawn_step2_file_update(app_handle: AppHandle) {
//     spawn(async move {
//     });
// }