use std::time::Duration;
use tauri::async_runtime::{spawn, JoinHandle};
use tauri::{AppHandle};
use crate::function::serial::{
    is_usb_tool_connected, get_current_data_density, serial_receive_clean, 
    serial_send, detect_serial_string, execute_command_and_wait};
use crate::function::dialog_test::{show_dialog_and_wait};
use crate::second_app::state::{AppStepStatus, AppTestStatus, 
    set_step_status, clean_step1_status, set_current_hardware, 
    set_target_serial, all_step_status_is_success, add_error_msg, get_error_msg};
use crate::function::printer::{generate_image_with_params, print_image, generate_defects_image_with_params, PRINTER_ENABLE, TARGET_PRINTER};
use crate::function::save::{get_config_str};


const NOT_CONNECTED_KVM_COUNT_THRESHOLD: u64 = 10;  // 未连接KVM超过10次，同步弹窗提示,约10s

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = true;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[app]{}", msg);
    }
}

pub fn spawn_app_step1_task(app_handle: AppHandle) -> JoinHandle<()> {
    spawn(async move {
        let mut app_step1_status = AppStepStatus::Unconnected;
        let mut current_step = app_step1_status.clone();
        let mut not_connected_kvm_count = 0;
        let mut target_name = String::new();
        let mut target_serial = String::new();
        let mut target_type = String::new();
        let mut wifi_exist = false;
        
        let _ = serial_receive_clean().await;
        loop {
            // 每轮一定要检测的内容：
            if !is_usb_tool_connected().await {
                app_step1_status = AppStepStatus::Unconnected;
            }
            // 需要以下两个事情：
            // 1. 记录变更，貌似不一定非要记录所有变更，只要是不同状态下进入ConnectedNoKVM就清零计数
            // 2. 如果未连接要再次连接
            // 3. 未知状态其实是暂态，进去一次很快会出来，所以不作为前端的状态变更和后端的时间计数
            if current_step != app_step1_status.clone() {
                if current_step != AppStepStatus::ConnectedNoKVM && 
                   current_step != AppStepStatus::Uncertain &&
                   app_step1_status == AppStepStatus::ConnectedNoKVM {
                    not_connected_kvm_count = 0;
                }
                current_step = app_step1_status.clone();
                log(&format!("应用步骤1状态变更: {:?}", current_step));
            }
            // 检测数据密度
            log(&format!("当前应用步骤1状态: {:?}", app_step1_status));
            match app_step1_status {
                AppStepStatus::Unconnected => {  // 未连接工具
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
                        app_step1_status = AppStepStatus::ConnectedNoKVM;
                        std::thread::sleep(Duration::from_secs(2));
                    }
                    continue;
                }
                AppStepStatus::ConnectedNoKVM => {  // 已连接工具, 未连接KVM
                    clean_step1_status(app_handle.clone());
                    set_step_status(app_handle.clone(), "wait_connection", AppTestStatus::Testing);

                    not_connected_kvm_count += 1;
                    if not_connected_kvm_count >= NOT_CONNECTED_KVM_COUNT_THRESHOLD {
                        // 未连接KVM超过10次，同步弹窗提示
                        set_step_status(app_handle.clone(), "wait_connection", AppTestStatus::Failed);
                        let response = show_dialog_and_wait(app_handle.clone(), "⚠️ 未检测到KVM，请检查KVM是否连接至测试工具".to_string(), vec![
                            serde_json::json!({ "text": "再次检测" }),
                            serde_json::json!({ "text": "确认连接无误，直接打印不良" })
                        ]);
                        if response == "确认连接无误，直接打印不良" {
                            log("用户选择了直接打印不良");
                            // 生成错误图片
                            add_error_msg("连接检测失败，可能是USB-C串口焊接不良/24P排线连接不良/eMMC固件错误 | ");

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
                            // 等待弹窗消失500ms
                            std::thread::sleep(Duration::from_millis(500));
                            break;
                        } else {
                            log("用户选择了重新检测连接");
                            not_connected_kvm_count = 0;
                            // 等待弹窗消失500ms
                            std::thread::sleep(Duration::from_millis(500));
                        }
                    }

                    log("已连接工具, 未连接KVM, 检测数据密度");
                    let current_data_density = get_current_data_density().await;
                    log(&format!("当前数据密度: {:?}", current_data_density));
                    if current_data_density == 0 {
                        app_step1_status = AppStepStatus::Uncertain;           // 进入不确定状态
                    } else {
                        app_step1_status = AppStepStatus::Booting;             // 进入开机中状态
                    }
                    continue;
                }
                AppStepStatus::Uncertain => {  // 状态不确定
                    log("状态不确定, 发送换行符");
                    serial_send("\n").await;
                    std::thread::sleep(Duration::from_millis(100));
                    let patterns = ["login", ":~#", "AXERA-UBOOT=>"];
                    let result = detect_serial_string(&patterns, 1000, 0).await;
                    log(&format!("检测结果: {}", result));
                    match result.as_str() {
                        "login" => {
                            log("检测到login, 进入开机中状态");
                            app_step1_status = AppStepStatus::BootedLogin;  // 已连接KVM，已开机（现在出现login）
                        }
                        ":~#" => {
                            log("检测到:~#, 进入已登录状态");
                            app_step1_status = AppStepStatus::CheckingState;  // 已连接KVM，已登录（现在出现:~#）
                        }
                        "AXERA-UBOOT=>" => {
                            log("检测到AXERA-UBOOT=>, 进入BOOT状态");
                            app_step1_status = AppStepStatus::Booted;  // 已连接KVM，不慎进入BOOT（现在出现AXERA-UBOOT=>，输入boot\n）
                        }
                        "UNMATCHED" => {
                            log("检测到UNMATCHED, 进入开机中状态");
                            app_step1_status = AppStepStatus::Booting;  // 已连接KVM，开机中
                        }
                        "NO-DATA" => {
                            log("检测到NO-DATA, 进入未连接KVM状态");
                            app_step1_status = AppStepStatus::ConnectedNoKVM;  // 未连接KVM
                        }
                        _ => {
                            log(&format!("其他情况: {}", result));
                            app_step1_status = AppStepStatus::ConnectedNoKVM;  // 未连接KVM
                        }
                    }
                }
                AppStepStatus::Booted => {  // 已连接KVM，不慎进入BOOT（现在出现AXERA-UBOOT=>，输入boot\n）
                    log("已连接KVM，不慎进入BOOT（现在出现AXERA-UBOOT=>，输入boot\n）, 发送boot\n");
                    set_step_status(app_handle.clone(), "wait_connection", AppTestStatus::Success);
                    set_step_status(app_handle.clone(), "wait_power_on", AppTestStatus::Testing);
                    serial_send("boot\n").await;
                    std::thread::sleep(Duration::from_millis(100));
                    app_step1_status = AppStepStatus::ConnectedNoKVM;  // 未连接KVM
                }
                AppStepStatus::Booting => {  // 已连接KVM，开机中
                    log("已连接KVM，开机中, 等待login...");
                    set_step_status(app_handle.clone(), "wait_connection", AppTestStatus::Success);
                    set_step_status(app_handle.clone(), "wait_power_on", AppTestStatus::Testing);
                    let patterns = ["login"];
                    let result = detect_serial_string(&patterns, 30000, 10).await;
                    match result.as_str() {
                        "login" => {
                            app_step1_status = AppStepStatus::BootedLogin;  // 已连接KVM，已开机（现在出现login）
                        }
                        "UNMATCHED" => {
                            // 开着，但是超时了，也有可能关不上了，建议是拔掉再试一次，或者打印贴纸
                            // ##
                        }
                        "NO-DATA" => {
                            app_step1_status = AppStepStatus::ConnectedNoKVM;  // 未连接KVM
                        }
                        "LOW-DENSITY" => {
                            log("等待过程数据密度低于限额, 进入未连接KVM状态");
                            app_step1_status = AppStepStatus::ConnectedNoKVM;  // 未连接KVM
                        }
                        _ => {
                            app_step1_status = AppStepStatus::ConnectedNoKVM;  // 未连接KVM
                        }
                    }
                }
                AppStepStatus::BootedLogin => {  // 已连接KVM，已开机（现在出现login）
                    log("已连接KVM，已开机（现在出现login）, 输入root密码");
                    set_step_status(app_handle.clone(), "wait_connection", AppTestStatus::Success);
                    set_step_status(app_handle.clone(), "wait_power_on", AppTestStatus::Testing);
                    if ! execute_command_and_wait("root\n", "Password", 1000).await {
                        app_step1_status = AppStepStatus::ConnectedNoKVM;  // 未连接KVM
                        continue;
                    }
                    log("输入root密码成功, 输入sipeed密码");
                    if ! execute_command_and_wait("sipeed\n", "Welcome", 1000).await {
                        app_step1_status = AppStepStatus::ConnectedNoKVM;  // 未连接KVM
                        continue;
                    }
                    // 等待一部分初始信息
                    std::thread::sleep(Duration::from_millis(100));
                    log("登录成功, 发送回车等待:~#");
                    if ! execute_command_and_wait("\n", ":~#", 1000).await { 
                        app_step1_status = AppStepStatus::ConnectedNoKVM;  // 未连接KVM
                        continue;
                    }
                    app_step1_status = AppStepStatus::CheckingState;  // 已连接KVM，已登录（现在出现:~#）
                }
                AppStepStatus::CheckingState => {  // 检查状态中，当前已登录（现在出现:~#）
                    log("已连接KVM，已登录（现在出现:~#）");
                    set_step_status(app_handle.clone(), "wait_connection", AppTestStatus::Success);
                    set_step_status(app_handle.clone(), "wait_power_on", AppTestStatus::Success);
                    set_step_status(app_handle.clone(), "get_status", AppTestStatus::Testing);
                    
                    // 是否产测
                    if !execute_command_and_wait("lsmod | grep 6911\n", "manage", 1000).await { 
                        // 未过测试，弹窗提示
                        let _ = show_dialog_and_wait(app_handle.clone(), "⚠️ 没有产测，请先过产测，然后回到ConnectedNoKVM".to_string(), vec![
                            serde_json::json!({ "text": "已拔出USB测试工具" })
                        ]);
                        // 等待弹窗消失500ms
                        std::thread::sleep(Duration::from_millis(500));
                        app_step1_status = AppStepStatus::ConnectedNoKVM;  // 未连接KVM
                        continue;
                    }

                    // 检查是否存在wifi
                    if execute_command_and_wait("ip a | grep wlan\n", "wlan0", 1000).await { 
                        wifi_exist = true;
                    } else {
                        log("未检测到wifi");
                    }

                    // 是否有串号，并获取串号
                    let mut serial_exit = false;
                    log("check serial & type");
                    // serial_send("cat /proc/lt6911_info/version\n").await;
                    let _ = execute_command_and_wait("cat /proc/lt6911_info/version\n", "version", 1000).await;
                    std::thread::sleep(Duration::from_secs(1));
                    let received = serial_receive_clean().await;
                    if !received.contains("Unknown") {
                        serial_exit = true;
                        if let Some(start) = received.find("(") {
                            if let Some(end) = received.find(")") {
                                target_type = received[start+1..end].to_string();
                                log(&format!("target_type: {}", target_type));
                            }
                        }
                        if let Some(start) = received.rfind("N") {
                            target_serial = received[start+0..start+9].to_string();
                            log(&format!("target_serial: {}", target_serial));
                        }
                        log("check serial & type end");
                    }
                    // 没有内容：
                    if !serial_exit {
                        let mut tmp_serial = String::new();
                        // 从cat /etc/test-kvm/serial 中获取串号，Nebc1000A
                        // serial_send("cat /etc/test-kvm/serial\n").await;
                        let _ = execute_command_and_wait("cat /etc/test-kvm/serial\n", "test-kvm", 1000).await;
                        std::thread::sleep(Duration::from_millis(100));     // 等待版本信息返回
                        let received = serial_receive_clean().await;
                        // 没有内容，或者内容中包含directory，都认为是失败
                        if !received.contains("No such file or directory") {
                            if let Some(start) = received.rfind("N") {
                                tmp_serial = received[start+0..start+9].to_string();
                                target_serial = tmp_serial.clone();
                                log(&format!("tmp_serial: {}", tmp_serial));
                            }
                        } else {
                            // 直接打印错误贴纸
                            add_error_msg("产测异常，请重新烧录镜像后重新产测");
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
                            // 弹窗提示没有产测，请先过产测，然后回到ConnectedNoKVM
                            let _ = show_dialog_and_wait(app_handle.clone(), "产测异常，请粘贴不良贴纸并拔出USB测试工具".to_string(), vec![
                                serde_json::json!({ "text": "已拔出USB测试工具" })
                            ]);
                            // 等待弹窗消失500ms
                            std::thread::sleep(Duration::from_millis(500));
                            app_step1_status = AppStepStatus::ConnectedNoKVM;  // 未连接KVM
                            continue;
                        }

                        // 弹窗获取ATX/Desk版本
                        let response = show_dialog_and_wait(app_handle.clone(), "⚠️ 请确认ATX/Desk版本".to_string(), vec![
                            serde_json::json!({ "text": "ATX" }),
                            serde_json::json!({ "text": "Desk" })
                        ]);
                        let tmp_type = if response == "ATX" { "ATX".to_string() } else { "Desk".to_string() };
                        // 等待弹窗消失500ms
                        std::thread::sleep(Duration::from_millis(500));

                        log(&format!("tmp_type: {:?}", tmp_type));  
                        // 从配置文件中获取批次代号（G）
                        let tmp_hardware_version = get_config_str("testing", "board_version").unwrap_or_default();
                        log(&format!("tmp_hardware_version: {:?}", tmp_hardware_version));
                        // 组合为完整信息，比如NanoKVM_Pro (Desk-G) Nebc1000A，echo写入到/proc/lt6911_info/version
                        let version_info = format!("NanoKVM_Pro ({}-{}) {}", tmp_type, tmp_hardware_version, tmp_serial);
                        if !execute_command_and_wait(&format!("echo \"{}\" > /proc/lt6911_info/version\n", version_info), "#", 2000).await { 
                            // #
                        }
                        // 组合为target_type
                        target_type = format!("{}-{}", tmp_type, tmp_hardware_version);
                    }

                    // 生成target_name
                    target_name = format!("NanoKVM-{}", target_type);

                    // 推送target_type到前端
                    set_current_hardware(app_handle.clone(), &target_type);
                    // 推送target_serial到前端
                    set_target_serial(app_handle.clone(), &target_serial);

                    set_step_status(app_handle.clone(), "get_status", AppTestStatus::Success);
                    app_step1_status = AppStepStatus::CheckingHDMI;  // 检查HDMI
                }
                AppStepStatus::CheckingHDMI => {  // 检查HDMI
                    log("检查HDMI");
                    set_step_status(app_handle.clone(), "video_capture", AppTestStatus::Testing);
                    // 从cat /proc/lt6911_info/status 中获取状态，new res时即为正常，否则提示拔插HDMI弹窗（选择再次测试/记为不良）
                    loop {
                        if !execute_command_and_wait("cat /proc/lt6911_info/status\n", "new res", 2000).await { 
                            // 弹窗提示拔插HDMI
                            let response = show_dialog_and_wait(app_handle.clone(), "⚠️ 请拔插HDMI线".to_string(), vec![
                                serde_json::json!({ "text": "再次测试" }),
                                serde_json::json!({ "text": "记为不良" })
                            ]);
                            // 等待弹窗消失500ms
                            std::thread::sleep(Duration::from_millis(500));
                            if response == "再次测试" {
                                set_step_status(app_handle.clone(), "video_capture", AppTestStatus::Repairing);
                                continue;
                            } else {
                                add_error_msg("HDMI测试异常，请重新拔插HDMI线");
                                set_step_status(app_handle.clone(), "video_capture", AppTestStatus::Failed);
                                app_step1_status = AppStepStatus::CheckingWiFi;
                                break;
                            }
                        } else {
                            break;
                        }
                    }

                    if app_step1_status == AppStepStatus::CheckingHDMI {
                        set_step_status(app_handle.clone(), "video_capture", AppTestStatus::Success);
                        app_step1_status = AppStepStatus::CheckingWiFi;
                    }
                }
                AppStepStatus::CheckingWiFi => {  // 检查WiFi
                    log("检查WiFi");

                    set_step_status(app_handle.clone(), "wifi_exist", AppTestStatus::Testing);
                    if !execute_command_and_wait("ls /etc/test-kvm/wifi*\n", "wifi_exist", 1000).await {
                        set_step_status(app_handle.clone(), "wifi_exist", AppTestStatus::Hidden);
                        app_step1_status = AppStepStatus::CheckingTouch;
                        continue;
                    }

                    if wifi_exist {
                        set_step_status(app_handle.clone(), "wifi_exist", AppTestStatus::Success);
                    } else {
                        set_step_status(app_handle.clone(), "wifi_exist", AppTestStatus::Failed);
                        add_error_msg("WiFi异常，请使用修复卡修复 | ");
                    }
                    app_step1_status = AppStepStatus::CheckingTouch;
                }
                AppStepStatus::CheckingTouch => {  // 检查触摸/屏幕
                    log("测试触摸/屏幕");
                    set_step_status(app_handle.clone(), "touch", AppTestStatus::Testing);
                    
                    if target_type.contains("ATX") {
                        let response = show_dialog_and_wait(app_handle.clone(), "请检查OLED是否正常显示".to_string(), vec![
                            serde_json::json!({ "text": "正常" }),
                            serde_json::json!({ "text": "异常，记录不良" })
                        ]);
                        // 等待弹窗消失500ms
                        std::thread::sleep(Duration::from_millis(500));
                        if response == "正常" {
                            set_step_status(app_handle.clone(), "touch", AppTestStatus::Success);
                        } else {
                            set_step_status(app_handle.clone(), "touch", AppTestStatus::Failed);
                            add_error_msg("OLED异常，可能是OLED排线连接问题，请更换OLED模块 | ");
                        }
                        set_step_status(app_handle.clone(), "knob", AppTestStatus::Hidden);
                        app_step1_status = AppStepStatus::Finished;
                    } else {
                        let response = show_dialog_and_wait(app_handle.clone(), "请检查触摸/屏幕是否正常，不要点到最后一步".to_string(), vec![
                            serde_json::json!({ "text": "正常" }),
                            serde_json::json!({ "text": "异常，记录不良" })
                        ]);
                        // 等待弹窗消失500ms
                        std::thread::sleep(Duration::from_millis(500));
                        if response == "正常" {
                            set_step_status(app_handle.clone(), "touch", AppTestStatus::Success);
                        } else {
                            set_step_status(app_handle.clone(), "touch", AppTestStatus::Failed);
                            add_error_msg("触摸屏异常，可能是排线连接问题 | ");
                        }
                        app_step1_status = AppStepStatus::CheckingKnob;
                    }
                }
                AppStepStatus::CheckingKnob => {  // 检查旋钮
                    log("测试旋钮");
                    set_step_status(app_handle.clone(), "knob", AppTestStatus::Testing);
                    // 弹窗提示检查旋钮
                    let response = show_dialog_and_wait(app_handle.clone(), "请检查旋钮是否正常".to_string(), vec![
                        serde_json::json!({ "text": "正常" }),
                        serde_json::json!({ "text": "异常，记录不良" })
                    ]);
                    // 等待弹窗消失500ms
                    std::thread::sleep(Duration::from_millis(500));
                    if response == "正常" {
                        set_step_status(app_handle.clone(), "knob", AppTestStatus::Success);
                    } else {
                        set_step_status(app_handle.clone(), "knob", AppTestStatus::Failed);
                        add_error_msg("旋钮测试异常，可能是端子线脱落");
                    }
                    app_step1_status = AppStepStatus::Finished;
                }
                AppStepStatus::Finished => {  // 完成
                    let _ = execute_command_and_wait("rm /etc/kvm/kvm_ui.toml\n", "#", 1000).await;
                    let _ = execute_command_and_wait("sync\n", "#", 1000).await;
                    log("测试完成");
                    set_step_status(app_handle.clone(), "print_label", AppTestStatus::Testing);
                    // 弹窗提示是否打印身份标签
                    let response = show_dialog_and_wait(app_handle.clone(), "是否打印身份标签".to_string(), vec![
                        serde_json::json!({ "text": "是" }),
                        serde_json::json!({ "text": "否" })
                    ]);
                    // 等待弹窗消失500ms
                    std::thread::sleep(Duration::from_millis(500));
                    if response == "是" {
                        // 打印身份标签
                        log("打印机已连接");
                        let img = generate_image_with_params(&target_serial, &target_name, wifi_exist);
                        if PRINTER_ENABLE {
                            if let Err(e) = print_image(&img, Some(TARGET_PRINTER)) {
                                log(&format!("打印图像失败: {}", e));
                                // #
                            }
                        }
                    }

                    if all_step_status_is_success() {
                        let _ = show_dialog_and_wait(app_handle.clone(), "测试完成，请拔出线缆".to_string(), vec![
                            serde_json::json!({ "text": "已拔出线缆" }),
                        ]);
                    } else {
                        // 打印错误贴纸
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
                        let _ = show_dialog_and_wait(app_handle.clone(), "测试过程中出现错误，请粘贴错误贴纸，并拔出线缆".to_string(), vec![
                            serde_json::json!({ "text": "已拔出线缆" }),
                        ]);
                    }
                    set_step_status(app_handle.clone(), "print_label", AppTestStatus::Success);
                    // 等待弹窗消失500ms
                    std::thread::sleep(Duration::from_millis(500));
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    })
}
