use std::thread;
use std::time::Duration;
use tauri::async_runtime::spawn;
use tauri::{AppHandle, Emitter};
use tokio::time::sleep;
use crate::threads::serial::{
    is_usb_tool_connected, get_current_data_density, 
    serial_send, detect_serial_string, wait_for_serial_data, execute_command_and_wait};
use crate::threads::dialog_test::{show_dialog_and_wait};
use lazy_static::lazy_static;
use crate::threads::update_state::{AppStep1Status, AppTestStatus, 
    set_step_status, clean_step1_status, set_target_ip, set_current_hardware, set_target_serial};
use tauri::async_runtime::JoinHandle;
use crate::threads::server::spawn_file_server_task;
use crate::threads::ssh::{ssh_execute_command, ssh_execute_command_check_success};
use crate::threads::save::{get_config_str};
use crate::threads::printer::{is_printer_connected, generate_image_with_params, print_image, PRINTER_ENABLE, TARGET_PRINTER};
use crate::threads::hdmi::if_two_monitor;
use crate::threads::camera::{get_camera_status, CameraStatus};

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
    log("进入step2_file_update");
    spawn(async move {
        log("更新KVM文件");
        // dtb
        set_step_status(app_handle.clone(), "dtb", AppTestStatus::Testing);
        let mut dtb_update_success = false;

        while !dtb_update_success {
            log("dtb文件更新中...");
            dtb_update_success = ssh_execute_command_check_success("/root/NanoKVM_Pro_Testing/test_sh/04_update_file.sh dtb", "dtb done").await.map(|(success, _)| success).unwrap_or(false);
            set_step_status(app_handle.clone(), "dtb", AppTestStatus::Repairing);
        }
        set_step_status(app_handle.clone(), "dtb", AppTestStatus::Success);

        // uboot
        set_step_status(app_handle.clone(), "uboot", AppTestStatus::Testing);
        let mut uboot_update_success = false;

        while !uboot_update_success {
            log("uboot文件更新中...");
            uboot_update_success = ssh_execute_command_check_success("/root/NanoKVM_Pro_Testing/test_sh/04_update_file.sh uboot", "uboot done").await.map(|(success, _)| success).unwrap_or(false);
            set_step_status(app_handle.clone(), "uboot", AppTestStatus::Repairing);
        }
        set_step_status(app_handle.clone(), "uboot", AppTestStatus::Success);

        // kernel
        set_step_status(app_handle.clone(), "kernel", AppTestStatus::Testing);
        let mut kernel_update_success = false;

        while !kernel_update_success {
            log("kernel文件更新中...");
            kernel_update_success = ssh_execute_command_check_success("/root/NanoKVM_Pro_Testing/test_sh/04_update_file.sh kernel", "kernel done").await.map(|(success, _)| success).unwrap_or(false);
            set_step_status(app_handle.clone(), "kernel", AppTestStatus::Repairing);
        }
        set_step_status(app_handle.clone(), "kernel", AppTestStatus::Success);

        // app
        set_step_status(app_handle.clone(), "app_install", AppTestStatus::Testing);
        let mut app_update_success = false;

        while !app_update_success {
            log("app文件更新中...");
            app_update_success = ssh_execute_command_check_success("/root/NanoKVM_Pro_Testing/test_sh/04_update_file.sh app", "app done").await.map(|(success, _)| success).unwrap_or(false);
            set_step_status(app_handle.clone(), "app_install", AppTestStatus::Repairing);
        }
        set_step_status(app_handle.clone(), "app_install", AppTestStatus::Success);
        loop {
            // log("sleep");
            sleep(Duration::from_secs(1)).await;
        }
    });
}

pub fn spawn_step2_hdmi_testing(app_handle: AppHandle) {
    log("进入step2_hdmi_testing");
    spawn(async move {
        let mut lt6911_rst_io = true;
        let mut lt86102_rst_io = true;
        let mut lt86102_rx_io = true;
        let mut lt86102_tx_io = true;
        let mut lt6911_int_io = true;
        let mut lt6911_i2c_io = true;
        let mut lt86102_i2c_io = true;

        log("HDMI测试中...");
        // hdmi_wait_connection
        set_step_status(app_handle.clone(), "hdmi_wait_connection", AppTestStatus::Testing);
        // 测试时注释
        // while !if_two_monitor() {
        //     log("HDMI未连接到第二显示器，等待中...");
        //     sleep(Duration::from_secs(1)).await;
        // }
        set_step_status(app_handle.clone(), "hdmi_wait_connection", AppTestStatus::Success);

        // hdmi_io_test
        set_step_status(app_handle.clone(), "hdmi_io_test", AppTestStatus::Testing);
        // ssh test hdmi io & if not pass print output
        let (hdmi_io_test_success, output) = ssh_execute_command_check_success("/root/NanoKVM_Pro_Testing/test_sh/05_hdmi_io.sh", "HDMI IO test passed").await.unwrap_or((false, String::new()));
        if !hdmi_io_test_success {
            log(&format!("hdmi_io_test失败，输出: {}", output));
            if output.contains("LT86102 RST 引脚异常") { lt86102_rst_io = false; }
            if output.contains("LT6911 RST 引脚异常") { lt6911_rst_io = false; }
            if output.contains("LT86102 RX 引脚异常") { lt86102_rx_io = false; }
            if output.contains("LT86102 TX 引脚异常") { lt86102_tx_io = false; }
            if output.contains("LT6911 INT 引脚异常") { lt6911_int_io = false; }
            if output.contains("LT6911 I2C 引脚异常") { lt6911_i2c_io = false; }
            if output.contains("LT86102 I2C 引脚异常") { lt86102_i2c_io = false; }
            set_step_status(app_handle.clone(), "hdmi_io_test", AppTestStatus::Repairing);
        } else {
            log("hdmi_io_test成功");
            set_step_status(app_handle.clone(), "hdmi_io_test", AppTestStatus::Success);
        }

        // 测试环出
        set_step_status(app_handle.clone(), "hdmi_loop_test", AppTestStatus::Testing);
        while true {
            log("hdmi loop out 测试中...");
            let camera_status = get_camera_status().await;
            match camera_status {
                CameraStatus::HasImage => {
                    log("hdmi loop out 测试成功");
                    set_step_status(app_handle.clone(), "hdmi_loop_test", AppTestStatus::Success);
                    break;
                }
                CameraStatus::Connected => {
                    log("hdmi loop out 测试失败");
                    set_step_status(app_handle.clone(), "hdmi_loop_test", AppTestStatus::Failed);
                    break;
                }
                CameraStatus::Disconnected => {
                    log("hdmi loop out 测试失败，摄像头未连接");
                    // delay 1s
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
        
        loop {
            log("sleep");
            sleep(Duration::from_secs(1)).await;
        }
    });
}

// pub fn spawn_step2_file_update(app_handle: AppHandle) {
//     spawn(async move {
//     });
// }