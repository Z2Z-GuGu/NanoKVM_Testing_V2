use std::time::Duration;
use tauri::async_runtime::{spawn, JoinHandle};
use tauri::AppHandle;
use tokio::time::sleep;
use crate::threads::update_state::{AppTestStatus, set_step_status, all_step_status_is_success, add_error_msg, get_error_msg};
use crate::threads::ssh::{ssh_execute_command_check_success, ssh_execute_command};
use crate::threads::camera::{get_camera_status, CameraStatus};
use crate::threads::save::{get_config_str, set_test_status};
use crate::threads::dialog_test::{show_dialog_and_wait};
use crate::threads::printer::{generate_defects_image_with_params, print_image, PRINTER_ENABLE, TARGET_PRINTER};
use crate::threads::hdmi::if_two_monitor;

const HDMI_IO_TEST_MAX_RETRY_COUNT: u64 = 5;
const HDMI_VIN_TEST_MAX_RETRY_COUNT: u64 = 5;
const HDMI_VERSION_TEST_MAX_RETRY_COUNT: u64 = 1;
const HDMI_EDID_TEST_MAX_RETRY_COUNT: u64 = 3;
const USB_TEST_MAX_RETRY_COUNT: u64 = 5;
const ETH_DOWNLOAD_TEST_MAX_RETRY_COUNT: u64 = 5;
const ETH_UPLOAD_TEST_MAX_RETRY_COUNT: u64 = 5;
const WIFI_CONNECT_MAX_RETRY_COUNT: u64 = 5;
const IO_TEST_MAX_RETRY_COUNT: u64 = 5;

// 枚举atx/desk：
#[derive(PartialEq, Clone)]
pub enum HardwareType {
    Atx,
    Desk,
}

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

pub fn spawn_step2_file_update(app_handle: AppHandle) -> JoinHandle<()> {
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
        
        // 等待1秒，确保文件更新完成
        sleep(Duration::from_secs(1)).await;
    })
}

pub fn spawn_step2_app_install(app_handle: AppHandle) -> JoinHandle<()> {
    log("进入step2_app_install");
    spawn(async move {
        // app
        set_step_status(app_handle.clone(), "app_install", AppTestStatus::Testing);
        let mut app_update_success = false;

        while !app_update_success {
            log("app文件更新中...");
            app_update_success = ssh_execute_command_check_success("/root/NanoKVM_Pro_Testing/test_sh/04_update_file.sh app", "app done").await.map(|(success, _)| success).unwrap_or(false);
            set_step_status(app_handle.clone(), "app_install", AppTestStatus::Repairing);
        }
        set_step_status(app_handle.clone(), "app_install", AppTestStatus::Success);
        
        // 等待1秒，确保文件更新完成
        sleep(Duration::from_secs(1)).await;
    })
}

pub fn spawn_step2_hdmi_testing(app_handle: AppHandle, target_type: &str, target_serial: &str) -> JoinHandle<()> {
    // log("进入step2_hdmi_testing");
    let serial = target_serial.to_string();
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
        spawn(async {
            log("启动vin_test测试服务");
            let _ = ssh_execute_command("/root/NanoKVM_Pro_Testing/test_sh/05_hdmi_test.sh start").await;
            log("vin_test测试服务退出");
        });

        log("HDMI测试中...");
        // hdmi_wait_connection
        set_step_status(app_handle.clone(), "hdmi_wait_connection", AppTestStatus::Testing);
        // 测试时注释
        while !if_two_monitor() {
            log("HDMI未连接到第二显示器，等待中...");
            sleep(Duration::from_secs(1)).await;
        }
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

            let mut hdmi_io_error_msg = "HDMI-IO:".to_string();
            if !lt86102_rst_io { hdmi_io_error_msg.push_str("LT86102 RST "); }
            if !lt6911_rst_io { hdmi_io_error_msg.push_str("LT6911 RST "); }
            if !lt86102_rx_io { hdmi_io_error_msg.push_str("LT86102 RX "); }
            if !lt86102_tx_io { hdmi_io_error_msg.push_str("LT86102 TX "); }
            if !lt6911_int_io { hdmi_io_error_msg.push_str("LT6911 INT "); }
            if !lt6911_i2c_io { hdmi_io_error_msg.push_str("LT6911 I2C "); }
            if !lt86102_i2c_io { hdmi_io_error_msg.push_str("LT86102 I2C "); }
            hdmi_io_error_msg.push_str(" | ");
            add_error_msg(&hdmi_io_error_msg);
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
                    if lt86102_rx_io && lt86102_tx_io && lt86102_i2c_io {
                        let _ = set_test_status(&serial, "lt86102", "Normal");
                    } else {
                        let _ = set_test_status(&serial, "lt86102", "Damage");
                    }
                    break;
                }
                CameraStatus::Connected => {
                    log("hdmi loop out 测试失败");
                    set_step_status(app_handle.clone(), "hdmi_loop_test", AppTestStatus::Failed);
                    add_error_msg("HDMI环出异常 | ");
                    let _ = set_test_status(&serial, "lt86102", "Damage");
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
        let (hdmi_capture_test_result, hdmi_capture_test_output) = auto_test_with_retry(&app_handle, "hdmi_capture_test", "/root/NanoKVM_Pro_Testing/test_sh/05_hdmi_test.sh vin", "HDMI VIN test passed", HDMI_VIN_TEST_MAX_RETRY_COUNT).await;
        if !hdmi_capture_test_result {
            log(&format!("hdmi_capture_test失败，输出: {}", hdmi_capture_test_output));
            add_error_msg("HDMI采集异常，建议检查IO错误或MIPI-CSI | ");
            let _ = set_test_status(&serial, "lt6911", "Damage");
        } else {
            if lt6911_int_io && lt6911_i2c_io && lt6911_rst_io {
                let _ = set_test_status(&serial, "lt6911", "Normal");
            } else {
                let _ = set_test_status(&serial, "lt6911", "Damage");
            }
        }

        // 写入version
        let full_version_str = format!("{}{}", target_type, target_serial);
        let (hdmi_version_test_result, hdmi_version_test_output) = auto_test_with_retry(&app_handle, "hdmi_version", &format!("/root/NanoKVM_Pro_Testing/test_sh/05_hdmi_test.sh version \"{}\"", full_version_str), "HDMI version write passed", HDMI_VERSION_TEST_MAX_RETRY_COUNT).await;
        if !hdmi_version_test_result {
            log(&format!("hdmi_version_test失败，输出: {}", hdmi_version_test_output));
            add_error_msg("Ver写入异常，建议检查6911 I2C | ");
        }

        // 写入EDID
        let (hdmi_write_edid_test_result, hdmi_write_edid_test_output) = auto_test_with_retry(&app_handle, "hdmi_write_edid", "/root/NanoKVM_Pro_Testing/test_sh/05_hdmi_test.sh edid", "HDMI EDID write passed", HDMI_EDID_TEST_MAX_RETRY_COUNT).await;
        if !hdmi_write_edid_test_result {
            log(&format!("hdmi_write_edid_test失败，输出: {}", hdmi_write_edid_test_output));
            add_error_msg("EDID写入异常，建议检查6911 I2C | ");
        }
        
        // 等待1秒，确保EDID写入完成
        sleep(Duration::from_secs(1)).await;
    })
}

pub fn spawn_step2_usb_testing(app_handle: AppHandle, target_serial: &str) -> JoinHandle<()> {
    let serial = target_serial.to_string();
    spawn(async move {
        log("USB测试中...");
        let (usb_test_result, usb_test_output) = auto_test_with_retry(&app_handle, "usb_wait_connection", "/root/NanoKVM_Pro_Testing/test_sh/06_usb_test.sh", "USB test passed", USB_TEST_MAX_RETRY_COUNT).await;
        if !usb_test_result {
            log(&format!("usb_test失败，输出: {}", usb_test_output));
            add_error_msg("USB测试异常，检查接口短路/24P排线连接/共模电感 | ");
            let _ = set_test_status(&serial, "usb", "Damage");
        } else {
            let _ = set_test_status(&serial, "usb", "Normal");
        }
        // 等待1秒，确保USB测试完成
        sleep(Duration::from_secs(1)).await;
    })
}

pub fn spawn_step2_eth_testing(app_handle: AppHandle, target_serial: &str, ip: &str) -> JoinHandle<()> {
    let serial = target_serial.to_string();
    let ip = ip.to_string();
    spawn(async move {
        log("网络测试中...");
        set_step_status(app_handle.clone(), "eth_wait_connection", AppTestStatus::Success);

        // 获取阈值
        let upload_speed_threshold = get_config_str("testing", "eth_up_speed").unwrap_or("300".to_string());
        let download_speed_threshold = get_config_str("testing", "eth_down_speed").unwrap_or("500".to_string());
        
        // 测试命令
        let upload_test_cmd = format!("/root/NanoKVM_Pro_Testing/test_sh/07_eth_test.sh upload {} \"http://{}:8080/upload\"", upload_speed_threshold, ip);
        let download_test_cmd = format!("/root/NanoKVM_Pro_Testing/test_sh/07_eth_test.sh download {} \"http://{}:8080/download_small\"", download_speed_threshold, ip);

        log(&format!("eth上传测试命令：{}", upload_test_cmd));
        log(&format!("eth下载测试命令：{}", download_test_cmd));
        
        // 测试上传
        let (eth_upload_test_result, eth_upload_test_output) = auto_test_with_retry(&app_handle, "eth_upload_test", &upload_test_cmd, "ETH upload test passed", ETH_UPLOAD_TEST_MAX_RETRY_COUNT).await;
        if !eth_upload_test_result {
            log(&format!("eth_upload_test失败，输出: {}", eth_upload_test_output));
        }
        // 测试下载
        let (eth_download_test_result, eth_download_test_output) = auto_test_with_retry(&app_handle, "eth_download_test", &download_test_cmd, "ETH download test passed", ETH_DOWNLOAD_TEST_MAX_RETRY_COUNT).await;
        if !eth_download_test_result {
            log(&format!("eth_download_test失败，输出: {}", eth_download_test_output));
        }

        if !eth_upload_test_result || !eth_download_test_result {
            add_error_msg("以太网网速异常 | ");
            let _ = set_test_status(&serial, "eth", "Damage");
        } else {
            let _ = set_test_status(&serial, "eth", "Normal");
        }

        // 等待1秒，确保ETH测试完成
        sleep(Duration::from_secs(1)).await;
    })
}

pub fn spawn_step2_wifi_testing(app_handle: AppHandle, target_serial: &str, ssid: &str, password: &str, wifi_exist: bool) -> JoinHandle<()> {
    let serial = target_serial.to_string();
    let ssid = ssid.to_string();
    let password = password.to_string();
    spawn(async move {
        if !wifi_exist {
            set_step_status(app_handle.clone(), "wifi_wait_connection", AppTestStatus::Hidden);
            set_step_status(app_handle.clone(), "wifi_upload_test", AppTestStatus::Hidden);
            set_step_status(app_handle.clone(), "wifi_download_test", AppTestStatus::Hidden);
            let _ = set_test_status(&serial, "wifi", "No hardware");
        } else {
            log("等待wifi连接...");
            let connect_test_cmd = format!("/root/NanoKVM_Pro_Testing/test_sh/08_wifi_test.sh connect {} {}", ssid, password);
            let (wifi_connect_result, wifi_connect_output) = auto_test_with_retry(&app_handle, "wifi_wait_connection", &connect_test_cmd, "WiFi connect passed", WIFI_CONNECT_MAX_RETRY_COUNT).await;
            if wifi_connect_result {
                // 连接成功
                let mut target_ip = String::new();
                if let Some(start) = wifi_connect_output.find("DHCP服务器IP: ") {
                    let content_start = start + "DHCP服务器IP: ".len();
                    let remaining = &wifi_connect_output[content_start..];
                    if let Some(end) = remaining.find('\n') {
                        let ip = &remaining[..end].trim();
                        log(&format!("RUST检测到当前板卡的IP为: {}", ip));
                        target_ip = ip.to_string();
                    }
                }
                // 获取阈值
                let upload_speed_threshold = get_config_str("testing", "wifi_up_speed").unwrap_or("10".to_string());
                let download_speed_threshold = get_config_str("testing", "wifi_down_speed").unwrap_or("10".to_string());
                
                // 测试命令
                let upload_test_cmd = format!("/root/NanoKVM_Pro_Testing/test_sh/08_wifi_test.sh upload {} \"http://{}:8080/upload\"", upload_speed_threshold, target_ip);
                let download_test_cmd = format!("/root/NanoKVM_Pro_Testing/test_sh/08_wifi_test.sh download {} \"http://{}:8080/download_small\"", download_speed_threshold, target_ip);

                log(&format!("wifi上传测试命令：{}", upload_test_cmd));
                log(&format!("wifi下载测试命令：{}", download_test_cmd));
                
                // 测试上传
                let (wifi_upload_test_result, wifi_upload_test_output) = auto_test_with_retry(&app_handle, "wifi_upload_test", &upload_test_cmd, "WiFi upload test passed", ETH_UPLOAD_TEST_MAX_RETRY_COUNT).await;
                if !wifi_upload_test_result {
                    log(&format!("wifi_upload_test失败，输出: {}", wifi_upload_test_output));
                }
                // 测试下载
                let (wifi_download_test_result, wifi_download_test_output) = auto_test_with_retry(&app_handle, "wifi_download_test", &download_test_cmd, "WiFi download test passed", ETH_DOWNLOAD_TEST_MAX_RETRY_COUNT).await;
                if !wifi_download_test_result {
                    log(&format!("wifi_download_test失败，输出: {}", wifi_download_test_output));
                }
                if !wifi_upload_test_result || !wifi_download_test_result {
                    add_error_msg("WiFi网速异常，检查天线连接 | ");
                    let _ = set_test_status(&serial, "wifi", "Damage");
                } else {
                    let _ = set_test_status(&serial, "wifi", "Normal");
                }
            } else {
                log(&format!("wifi连接失败，输出: {}", wifi_connect_output));
                add_error_msg("WiFi连接异常，检查SDIO | ");
                let _ = set_test_status(&serial, "wifi", "Damage");
            }
        }
        // 等待1秒，确保WIFI测试完成
        sleep(Duration::from_secs(1)).await;
    })
}

pub fn spawn_step2_penal_testing(app_handle: AppHandle, hardware_type: HardwareType) -> JoinHandle<()> {
    spawn(async move {
        log("启动屏幕测试服务");
        if hardware_type == HardwareType::Atx {
            set_step_status(app_handle.clone(), "screen", AppTestStatus::Testing);
            let _ = ssh_execute_command("/root/NanoKVM_Pro_Testing/test_sh/09_panel_test.sh oled 60").await;
            log("屏幕测试服务退出");
        } else {
            set_step_status(app_handle.clone(), "screen", AppTestStatus::Testing);
            let _ = ssh_execute_command("/root/NanoKVM_Pro_Testing/test_sh/09_panel_test.sh lcd 60").await;
            log("屏幕测试服务退出");
        }

        // 等待1秒，确保屏幕测试完成
        sleep(Duration::from_secs(1)).await;
    })
}

pub fn spawn_step2_ux_testing(app_handle: AppHandle, target_serial: &str, hardware_type: HardwareType, auto_type: bool) -> JoinHandle<()> {
    let serial = target_serial.to_string();
    let need_set_fail = !auto_type;
    spawn(async move {
        // 等待屏幕闪烁程序启动
        sleep(Duration::from_secs(2)).await;
        // 弹窗主动判断测试是否完成
        let response = show_dialog_and_wait(app_handle.clone(), "请查看屏幕是否闪烁".to_string(), vec![
            serde_json::json!({ "text": "YES" }),
            serde_json::json!({ "text": "NO" })
        ]);
        if response == "NO" {
            set_step_status(app_handle.clone(), "screen", AppTestStatus::Failed);
            add_error_msg("屏幕不闪烁 | ");
            let _ = set_test_status(&serial, "screen", "Damage");
        } else {
            set_step_status(app_handle.clone(), "screen", AppTestStatus::Success);
            let _ = set_test_status(&serial, "screen", "Normal");
        }
        // 等待弹窗消失500ms
        // std::thread::sleep(Duration::from_millis(500));
        if hardware_type == HardwareType::Atx {
            let _ = ssh_execute_command("kill $(cat /tmp/oled.pid)").await;
            set_step_status(app_handle.clone(), "touch", AppTestStatus::Hidden);
            set_step_status(app_handle.clone(), "knob", AppTestStatus::Hidden);
            let _ = set_test_status(&serial, "touch", "No hardware");
            let _ = set_test_status(&serial, "rotary", "No hardware");
        } else {
            let _ = ssh_execute_command("kill $(cat /tmp/lcd.pid)").await;
            // 测试触摸
            if need_set_fail {
                // 需要直接给一个失败状态
                set_step_status(app_handle.clone(), "touch", AppTestStatus::Testing);
                add_error_msg("触摸 | ");
                let _ = set_test_status(&serial, "touch", "Damage");
            } else {
                // 正常的触摸测试
                let (touch_test_result, touch_test_output) = auto_test_with_retry(&app_handle, "touch", "/root/NanoKVM_Pro_Testing/test_sh/09_panel_test.sh touch 60", "Touch test passed", 1).await;
                if !touch_test_result {
                    log(&format!("touch测试失败，输出: {}", touch_test_output));
                    add_error_msg("触摸 | ");
                    let _ = set_test_status(&serial, "touch", "Damage");
                } else {
                    let _ = set_test_status(&serial, "touch", "Normal");
                }
            }

            // 测试旋钮
            let (knob_test_result, knob_test_output) = auto_test_with_retry(&app_handle, "knob", "/root/NanoKVM_Pro_Testing/test_sh/09_panel_test.sh rotary 60", "Rotary test passed", 1).await;
            if !knob_test_result {
                log(&format!("knob测试失败，输出: {}", knob_test_output));
                add_error_msg("旋钮 | ");
                let _ = set_test_status(&serial, "rotary", "Damage");
            } else {
                let _ = set_test_status(&serial, "rotary", "Normal");
            }
        }

        // 等待1秒，确保UX测试完成
        sleep(Duration::from_secs(1)).await;
    })
}

pub fn spawn_step2_atx_testing(app_handle: AppHandle, target_serial: &str) -> JoinHandle<()> {
    let serial = target_serial.to_string();
    spawn(async move {        
        let (atx_test_result, atx_test_output) = auto_test_with_retry(&app_handle, "atx", "/root/NanoKVM_Pro_Testing/test_sh/10_atx_test.sh desk", "ATX test passed", IO_TEST_MAX_RETRY_COUNT).await;
        if !atx_test_result {
            log(&format!("atx_test失败，输出: {}", atx_test_output));
            add_error_msg("ATX IO异常 | ");
            let _ = set_test_status(&serial, "atx", "Damage");
        } else {
            let _ = set_test_status(&serial, "atx", "Normal");
        }
        
        // 等待1秒，确保ATX测试完成
        sleep(Duration::from_secs(1)).await;
    })
}

pub fn spawn_step2_io_testing(app_handle: AppHandle, target_serial: &str) -> JoinHandle<()> {
    let serial = target_serial.to_string();
    spawn(async move {        
        let (io_test_result, io_test_output) = auto_test_with_retry(&app_handle, "io", "/root/NanoKVM_Pro_Testing/test_sh/11_io_test.sh 10", "IO test passed", IO_TEST_MAX_RETRY_COUNT).await;
        if !io_test_result {
            log(&format!("io_test失败，输出: {}", io_test_output));
            add_error_msg("WS2812 IO异常 | ");
            let _ = set_test_status(&serial, "ws2812", "Damage");
        } else {
            let _ = set_test_status(&serial, "ws2812", "Normal");
        }
        
        // 等待1秒，确保IO测试完成
        sleep(Duration::from_secs(1)).await;
    })
}

pub fn spawn_step2_tf_testing(app_handle: AppHandle, target_serial: &str) -> JoinHandle<()> {
    let serial = target_serial.to_string();
    spawn(async move {        
        let (tf_test_result, tf_test_output) = auto_test_with_retry(&app_handle, "tf_card", "/root/NanoKVM_Pro_Testing/test_sh/12_tf_test.sh", "TF test passed", IO_TEST_MAX_RETRY_COUNT).await;
        if !tf_test_result {
            log(&format!("tf_test失败，输出: {}", tf_test_output));
            add_error_msg("TF卡，检查SDIO相关器件/测试卡是否损坏 | ");
            let _ = set_test_status(&serial, "sdcard", "Damage");
        } else {
            let _ = set_test_status(&serial, "sdcard", "Normal");
        }
        
        // 等待1秒，确保TF测试完成
        sleep(Duration::from_secs(1)).await;
    })
}

pub fn spawn_step2_uart_testing(app_handle: AppHandle, target_serial: &str, hardware_type: HardwareType) -> JoinHandle<()> {
    let serial = target_serial.to_string();
    spawn(async move {
        if hardware_type == HardwareType::Desk {
            let (uart_test_result, uart_test_output) = auto_test_with_retry(&app_handle, "uart", "/root/NanoKVM_Pro_Testing/test_sh/13_uart_test.sh", "UART test passed", IO_TEST_MAX_RETRY_COUNT).await;
            if !uart_test_result {
                log(&format!("uart_test失败，输出: {}", uart_test_output));
                add_error_msg("UART异常，检查24P排线 | ");
                let _ = set_test_status(&serial, "uart", "Damage");
            } else {
                let _ = set_test_status(&serial, "uart", "Normal");
            }
        } else {
            set_step_status(app_handle.clone(), "uart", AppTestStatus::Hidden);
        }
        
        // 等待1秒，确保UART测试完成
        sleep(Duration::from_secs(1)).await;
    })
}

pub fn spawn_step3_test_end(app_handle: AppHandle, target_serial: &str) -> JoinHandle<()> {
    let serial = target_serial.to_string();
    spawn(async move {
        if all_step_status_is_success() {
            set_step_status(app_handle.clone(), "auto_start", AppTestStatus::Testing);
            let _ = ssh_execute_command("/root/NanoKVM_Pro_Testing/test_sh/14_test_end.sh").await;
            let _ = ssh_execute_command("rm -r /root/*").await;
            set_step_status(app_handle.clone(), "auto_start", AppTestStatus::Success);
            let _ = set_test_status(&serial, "app", "Normal");
            let _ = set_test_status(&serial, "test_pass", "true");
        } else {
            set_step_status(app_handle.clone(), "print_error_msg", AppTestStatus::Testing);
            // 打印错误信息
            let error_msg = get_error_msg();
            if !error_msg.is_empty() {
                log(&format!("测试过程中出现错误: {}", error_msg));
                // 生成错误图片
                let img = generate_defects_image_with_params(&error_msg);
                if PRINTER_ENABLE {
                    if let Err(e) = print_image(&img, Some(TARGET_PRINTER)) {
                        log(&format!("打印图像失败: {}", e));
                        // #
                    }
                }
            }
            set_step_status(app_handle.clone(), "print_error_msg", AppTestStatus::Success);
        }
        sleep(Duration::from_millis(100)).await;
    })
}

// pub fn spawn_step2_file_update(app_handle: AppHandle) {
//     spawn(async move {
//     });
// }
