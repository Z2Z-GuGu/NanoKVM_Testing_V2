use std::time::Duration;
use tauri::async_runtime::spawn;
use tauri::AppHandle;
use tokio::time::sleep;
use crate::threads::update_state::{AppTestStatus, set_step_status};
use crate::threads::ssh::{ssh_execute_command_check_success, ssh_execute_command};
use crate::threads::camera::{get_camera_status, CameraStatus};
use crate::threads::server::spawn_file_server_task;
use crate::threads::save::get_config_str;

const HDMI_IO_TEST_MAX_RETRY_COUNT: u64 = 5;
const HDMI_VIN_TEST_MAX_RETRY_COUNT: u64 = 5;
const HDMI_VERSION_TEST_MAX_RETRY_COUNT: u64 = 1;
const HDMI_EDID_TEST_MAX_RETRY_COUNT: u64 = 3;
const USB_TEST_MAX_RETRY_COUNT: u64 = 5;
const ETH_DOWNLOAD_TEST_MAX_RETRY_COUNT: u64 = 5;
const ETH_UPLOAD_TEST_MAX_RETRY_COUNT: u64 = 5;

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = true;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[step2]{}", msg);
    }
}

// 自动多次测试
async fn auto_test_with_retry(app_handle: &AppHandle, test_name: &str, test_cmd: &str, success_msg: &str, max_retry: u64) -> (bool, String) {
    let mut retry_count = 0;
    set_step_status(app_handle.clone(), test_name, AppTestStatus::Testing);
    let mut last_output = String::new();
    while retry_count < max_retry {
        log(&format!("{} 测试中...", test_name));
        let (success, output) = ssh_execute_command_check_success(test_cmd, success_msg).await.unwrap_or((false, String::new()));
        last_output = output.clone();
        if success {
            log(&format!("{} 成功", test_name));
            set_step_status(app_handle.clone(), test_name, AppTestStatus::Success);
            return (true, output);
        } else {
            set_step_status(app_handle.clone(), test_name, AppTestStatus::Repairing);
            log(&format!("{} 失败，输出: {}", test_name, output));
            retry_count += 1;
            sleep(Duration::from_secs(1)).await;
        }
    }
    set_step_status(app_handle.clone(), test_name, AppTestStatus::Failed);
    (false, last_output)
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

pub fn spawn_step2_hdmi_testing(app_handle: AppHandle, target_type: &str, target_serial: &str) {
    // log("进入step2_hdmi_testing");
    let target_type = target_type.to_string();
    let target_serial = target_serial.to_string();
    spawn(async move {
        let mut lt6911_rst_io: bool = true;
        let mut lt86102_rst_io: bool = true;
        let mut lt86102_rx_io: bool = true;
        let mut lt86102_tx_io: bool = true;
        let mut lt6911_int_io: bool = true;
        let mut lt6911_i2c_io: bool = true;
        let mut lt86102_i2c_io: bool = true;

        // 先启动vin_test
        spawn(async move {
            log("启动vin_test测试服务");
            let _ = ssh_execute_command("/root/NanoKVM_Pro_Testing/test_sh/05_hdmi_test.sh start").await;
            log("vin_test测试服务退出");
        });

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
        let (hdmi_io_test_result, hdmi_io_test_output) = auto_test_with_retry(&app_handle, "hdmi_io_test", "/root/NanoKVM_Pro_Testing/test_sh/05_hdmi_test.sh io", "HDMI IO test passed", HDMI_IO_TEST_MAX_RETRY_COUNT).await;
        if !hdmi_io_test_result {
            log(&format!("hdmi_io_test失败，输出: {}", hdmi_io_test_output));
            if hdmi_io_test_output.contains("LT86102 RST 引脚异常") { lt86102_rst_io = false; }
            if hdmi_io_test_output.contains("LT6911 RST 引脚异常") { lt6911_rst_io = false; }
            if hdmi_io_test_output.contains("LT86102 RX 引脚异常") { lt86102_rx_io = false; }
            if hdmi_io_test_output.contains("LT86102 TX 引脚异常") { lt86102_tx_io = false; }
            if hdmi_io_test_output.contains("LT6911 INT 引脚异常") { lt6911_int_io = false; }
            if hdmi_io_test_output.contains("LT6911 I2C 引脚异常") { lt6911_i2c_io = false; }
            if hdmi_io_test_output.contains("LT86102 I2C 引脚异常") { lt86102_i2c_io = false; }
        }

        if lt6911_rst_io && lt86102_rst_io && lt86102_rx_io && lt86102_tx_io && lt6911_int_io && lt6911_i2c_io && lt86102_i2c_io {
            log("所有引脚正常");
        }

        // 测试环出
        set_step_status(app_handle.clone(), "hdmi_loop_test", AppTestStatus::Testing);
        loop {
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

        // 测试采集
        let _ = auto_test_with_retry(&app_handle, "hdmi_capture_test", "/root/NanoKVM_Pro_Testing/test_sh/05_hdmi_test.sh vin", "HDMI VIN test passed", HDMI_VIN_TEST_MAX_RETRY_COUNT).await;

        // 写入version
        let full_version_str = format!("{}{}", target_type, target_serial);
        let _ = auto_test_with_retry(&app_handle, "hdmi_version", &format!("/root/NanoKVM_Pro_Testing/test_sh/05_hdmi_test.sh version \"{}\"", full_version_str), "HDMI version write passed", HDMI_VERSION_TEST_MAX_RETRY_COUNT).await;

        // 写入EDID
        let _ = auto_test_with_retry(&app_handle, "hdmi_write_edid", "/root/NanoKVM_Pro_Testing/test_sh/05_hdmi_test.sh edid", "HDMI EDID write passed", HDMI_EDID_TEST_MAX_RETRY_COUNT).await;
        
        loop {
            // log("sleep");
            sleep(Duration::from_secs(1)).await;
        }
    });
}

pub fn spawn_step2_usb_testing(app_handle: AppHandle) {
    spawn(async move {
        log("USB测试中...");
        let _ = auto_test_with_retry(&app_handle, "usb_wait_connection", "/root/NanoKVM_Pro_Testing/test_sh/06_usb_test.sh", "USB test passed", USB_TEST_MAX_RETRY_COUNT).await;
        sleep(Duration::from_secs(1)).await;
    });
}

pub fn spawn_step2_net_testing(app_handle: AppHandle, ip: &str) {
    let ip = ip.to_string();
    spawn(async move {
        log("网络测试中...");
        set_step_status(app_handle.clone(), "eth_wait_connection", AppTestStatus::Success);
        let handle = spawn_file_server_task();     // 启动文件服务器任务
        log("文件服务器任务已启动");
        // 获取阈值
        let upload_speed_threshold = get_config_str("testing", "eth_up_speed").unwrap_or("300".to_string());
        let download_speed_threshold = get_config_str("testing", "eth_down_speed").unwrap_or("500".to_string());
        
        // 测试命令
        let upload_test_cmd = format!("/root/NanoKVM_Pro_Testing/test_sh/07_eth_test.sh upload {} \"http://{}:8080/upload\"", upload_speed_threshold, ip);
        let download_test_cmd = format!("/root/NanoKVM_Pro_Testing/test_sh/07_eth_test.sh download {} \"http://{}:8080/download\"", download_speed_threshold, ip);

        log(&format!("上传测试命令：{}", upload_test_cmd));
        log(&format!("下载测试命令：{}", download_test_cmd));
        
        // 测试上传
        let _ = auto_test_with_retry(&app_handle, "eth_upload_test", &upload_test_cmd, "ETH upload test passed", ETH_UPLOAD_TEST_MAX_RETRY_COUNT).await;
        // 测试下载
        let _ = auto_test_with_retry(&app_handle, "eth_download_test", &download_test_cmd, "ETH download test passed", ETH_DOWNLOAD_TEST_MAX_RETRY_COUNT).await;

        handle.abort();
        sleep(Duration::from_secs(1)).await;
    });
}


// pub fn spawn_step2_file_update(app_handle: AppHandle) {
//     spawn(async move {
//     });
// }
