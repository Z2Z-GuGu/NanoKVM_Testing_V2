use std::time::Duration;
use tauri::async_runtime::{spawn, JoinHandle};
use tauri::{AppHandle};
use crate::function::serial::{
    is_usb_tool_connected, get_current_data_density, 
    serial_send, detect_serial_string, execute_command_and_wait};
use crate::function::dialog_test::{show_dialog_and_wait};
use crate::function::update_state::{AppStepStatus, AppTestStatus, 
    set_step_status, clean_step1_status, set_target_ip, set_current_hardware, 
    set_target_serial, all_step_status_is_success, add_error_msg, get_error_msg};
use crate::function::server::spawn_file_server_task;
use crate::function::ssh::{ssh_execute_command, ssh_execute_command_check_success};
use crate::function::save::{get_config_str, create_serial_number, set_test_status};
use crate::function::printer::{is_printer_connected, generate_image_with_params, print_image, generate_defects_image_with_params, PRINTER_ENABLE, TARGET_PRINTER};
use crate::test_app::step2::{spawn_step2_file_update, spawn_step2_hdmi_testing, 
    spawn_step2_usb_testing, spawn_step2_eth_testing, spawn_step2_wifi_testing, 
    spawn_step2_penal_testing, spawn_step2_ux_testing, spawn_step2_atx_testing,
    spawn_step2_io_testing, spawn_step2_tf_testing, spawn_step2_uart_testing, 
    HardwareType, spawn_step3_test_end, spawn_step2_app_install};
use crate::function::static_eth::STATIC_IP_ENABLE;

const NOT_CONNECTED_KVM_COUNT_THRESHOLD: u64 = 10;  // 未连接KVM超过10次，同步弹窗提示,约10s
const GET_IP_MAX_RETRY_COUNT: u64 = 10;
const DOWNLOAD_MAX_RETRY_COUNT: u64 = 5;

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = true;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[app]{}", msg);
    }
}

// lazy_static! {
//     pub static ref APP_STEP1_STATUS: Mutex<AppStepStatus> = Mutex::new(AppStepStatus::Unconnected);    // 应用步骤1状态全局变量
// }

pub fn spawn_app_step1_task(app_handle: AppHandle, ssid: String, password: String, static_ip: String, target_ip: String) -> JoinHandle<()> {
    spawn(async move {
        let current_static_ip = static_ip;
        let current_target_ip = target_ip;
        let ssid = ssid;
        let password = password;
        let mut app_step1_status = AppStepStatus::Unconnected;
        let mut current_step = app_step1_status.clone();
        let mut not_connected_kvm_count = 0;
        let mut target_serial = String::new();
        let mut target_name = String::new();
        let mut target_type = String::new();
        let mut wifi_exist = false;
        let mut target_hardware_type = HardwareType::Desk;
        let mut soc_id = String::new();
        let mut auto_type = true;
        let mut get_ip_retry_count = 0;
        let mut download_retry_count = 0;
        let file_server_handle = spawn_file_server_task();     // 启动文件服务器任务
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
                            app_step1_status = AppStepStatus::Finished;  // 跳转到结束
                            continue;
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
                            app_step1_status = AppStepStatus::LoggedIn;  // 已连接KVM，已登录（现在出现:~#）
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
                    set_step_status(app_handle.clone(), "wait_boot", AppTestStatus::Testing);
                    serial_send("boot\n").await;
                    std::thread::sleep(Duration::from_millis(100));
                    app_step1_status = AppStepStatus::ConnectedNoKVM;  // 未连接KVM
                }
                AppStepStatus::Booting => {  // 已连接KVM，开机中
                    log("已连接KVM，开机中, 等待login...");
                    set_step_status(app_handle.clone(), "wait_connection", AppTestStatus::Success);
                    set_step_status(app_handle.clone(), "wait_boot", AppTestStatus::Testing);
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
                    set_step_status(app_handle.clone(), "wait_boot", AppTestStatus::Testing);
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
                    app_step1_status = AppStepStatus::LoggedIn;  // 已连接KVM，已登录（现在出现:~#）
                }
                AppStepStatus::LoggedIn => {  // 已连接KVM，已登录（现在出现:~#）
                    log("已连接KVM，已登录（现在出现:~#）");
                    set_step_status(app_handle.clone(), "wait_connection", AppTestStatus::Success);
                    set_step_status(app_handle.clone(), "wait_boot", AppTestStatus::Success);
                    set_step_status(app_handle.clone(), "get_ip", AppTestStatus::Testing);
                    // 确保开启ssh服务
                    log("正在开启ssh服务...");
                    let _ = execute_command_and_wait("sudo systemctl start sshd.service\n", "#", 2000).await;
                    if STATIC_IP_ENABLE {
                        if ! execute_command_and_wait("pkill dhclient\n", ":~#", 1000).await { 
                            log("关闭DHCP超时");
                            // ##
                        }
                        // 清空ip
                        log("清空ip");
                        if ! execute_command_and_wait("ip addr flush dev eth0\n", ":~#", 1000).await {
                            log("清空ip超时");
                            // ##
                        };
                        // 设置静态IP
                        log(&format!("设置静态IP: {}", current_static_ip));
                        if ! execute_command_and_wait(&format!("ip addr add {} dev eth0\n", current_target_ip), ":~#", 1000).await { 
                            log("设置静态IP超时");
                            // ##
                        };
                        // 配置对方路由
                        log(&format!("配置对方路由: {}", current_static_ip));
                        if ! execute_command_and_wait(&format!("ip route add {} dev eth0\n", current_static_ip), ":~#", 1000).await { 
                            log("配置对方路由超时");
                            // ##
                        };
                        while ! execute_command_and_wait(&format!("ping -c 1 {}\n", current_static_ip), "1 received", 1000).await {
                            get_ip_retry_count += 1;
                            if get_ip_retry_count >= GET_IP_MAX_RETRY_COUNT {
                                log("获取IP超时");
                                set_step_status(app_handle.clone(), "get_ip", AppTestStatus::Failed);
                                // 弹窗选择打印不良/再次检测
                                let response = show_dialog_and_wait(app_handle.clone(), "获取IP失败，是否再检测一遍".to_string(), vec![
                                    serde_json::json!({ "text": "再次检测" }),
                                    serde_json::json!({ "text": "否，直接打印不良" })
                                ]);
                                if response == "否，直接打印不良" {
                                    log("用户选择了直接打印不良");
                                    // 生成错误图片
                                    add_error_msg("以太网连接异常，请检查网线连接或PHY部分焊接 | ");

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
                                    app_step1_status = AppStepStatus::Finished;  // 跳转到结束
                                    break;
                                } else {
                                    log("用户选择了YES，重新检测以太网");
                                    get_ip_retry_count = 0;
                                    // 等待弹窗消失500ms
                                    std::thread::sleep(Duration::from_millis(500));
                                }
                            }
                            set_step_status(app_handle.clone(), "get_ip", AppTestStatus::Repairing);
                            log("需要发送CTRL C确保退出ping");
                            if ! execute_command_and_wait("\x03", ":~#", 1000).await {
                                log("CTRL+C超时");
                                // ##
                            };
                            // 清空ip
                            log("清空ip");
                            if ! execute_command_and_wait("ip addr flush dev eth0\n", ":~#", 1000).await {
                                log("清空ip超时");
                                // ##
                            };
                            // 设置静态IP
                            log("设置静态IP");
                            if ! execute_command_and_wait(&format!("ip addr add {} dev eth0\n", current_target_ip), ":~#", 1000).await { 
                                log("设置静态IP超时");
                                // ##
                            };
                            // 配置对方路由
                            log(&format!("配置对方路由: {}", current_static_ip));
                            if ! execute_command_and_wait(&format!("ip route add {} dev eth0\n", current_static_ip), ":~#", 1000).await { 
                                log("配置对方路由超时");
                                // ##
                            };
                        }
                    }
                    set_step_status(app_handle.clone(), "get_ip", AppTestStatus::Success);
                    set_target_ip(app_handle.clone(), &current_target_ip);
                    if app_step1_status == AppStepStatus::Finished {
                        continue;
                    }
                    app_step1_status = AppStepStatus::DownloadFile;  // 下载文件中
                }
                AppStepStatus::DownloadFile => {  // 下载文件中
                    log("下载文件中");
                    set_step_status(app_handle.clone(), "download_test", AppTestStatus::Testing);

                    let (ls_success, _) = ssh_execute_command_check_success("ls /root/test.tar", "test.tar").await.unwrap_or((false, String::new()));
                    if ls_success {
                        // 找到了文件,弹窗是否重新下载
                        let response = show_dialog_and_wait(app_handle.clone(), "待测KVM中已存在产测文件，是否使用最新文件测试".to_string(), vec![
                            serde_json::json!({ "text": "YES" }),
                            serde_json::json!({ "text": "NO" })
                        ]);
                        if response == "NO" {
                            log("用户选择了不重新下载");
                            set_step_status(app_handle.clone(), "download_test", AppTestStatus::Success);
                            app_step1_status = AppStepStatus::CheckingHardware;  // 未连接KVM
                            continue;
                        } else {
                            log("用户选择了重新下载");
                        }
                    }

                    let _ = ssh_execute_command("rm -rf /root/NanoKVM_Pro_Testing").await;

                    loop {
                        download_retry_count += 1;
                        if download_retry_count > DOWNLOAD_MAX_RETRY_COUNT {
                            log("下载文件失败");
                            set_step_status(app_handle.clone(), "download_test", AppTestStatus::Failed);
                            app_step1_status = AppStepStatus::LoggedIn;  // 跳转到结束
                            break;
                        }
                        
                        let _ = ssh_execute_command("curl \"http://172.168.100.1:8080/download\" --output /root/test.tar -s -o /dev/null -w \"speed: %{speed_download} B/s\\n\"").await;

                        let (ls_success, _) = ssh_execute_command_check_success("ls /root/test.tar", "test.tar").await.unwrap_or((false, String::new()));
                        if ls_success {
                            break;
                        }

                        // 等待1秒
                        std::thread::sleep(Duration::from_secs(1));
                    }

                    if app_step1_status != AppStepStatus::DownloadFile {
                        continue;
                    }

                    // 解压
                    let _ = ssh_execute_command("tar -xf /root/test.tar -C /root/").await;

                    // 设置文件权限
                    let _ = ssh_execute_command("chmod -R +x /root/NanoKVM_Pro_Testing").await;
                    
                    set_step_status(app_handle.clone(), "download_test", AppTestStatus::Success);
                    app_step1_status = AppStepStatus::CheckingHardware;  // 检查硬件中
                }
                AppStepStatus::CheckingHardware => {  // 检查硬件中
                    log("检查硬件中");
                    set_step_status(app_handle.clone(), "detect_hardware", AppTestStatus::Testing);

                    match ssh_execute_command("/root/NanoKVM_Pro_Testing/test_sh/01_test_hardware.sh").await {
                        Ok(output) => {
                            // log(&format!("输出: \n{}", output));
                            // 判断是否存在：“弹窗内容”，获取"弹窗内容："到"\n"之间的内容
                            if let Some(start) = output.find("弹窗内容：") {
                                let content_start = start + "弹窗内容：".len();
                                let remaining = &output[content_start..];
                                if let Some(end) = remaining.find('\n') {
                                    let popup_content = &remaining[..end].trim();
                                    log(&format!("弹窗内容: {}", popup_content));
                                    // 弹窗
                                    let response = show_dialog_and_wait(app_handle.clone(), popup_content.to_string(), vec![
                                        serde_json::json!({ "text": "YES" }),
                                        serde_json::json!({ "text": "NO" })
                                    ]);
                                    if response == "NO" {
                                        log("用户选择了NO");
                                        // 获取哪些硬件已经通过产测
                                        // #
                                    } else {
                                        log("用户选择了YES，清除已测试硬件记录");
                                        let _ = ssh_execute_command("/root/NanoKVM_Pro_Testing/test_sh/02_rm_tested.sh").await.unwrap();
                                    }
                                    // 等待弹窗消失500ms
                                    std::thread::sleep(Duration::from_millis(500));
                                }
                            }
                            // 获取当前板卡的类型
                            if let Some(start) = output.find("当前板卡的类型为：") {
                                let content_start = start + "当前板卡的类型为：".len();
                                let remaining = &output[content_start..];
                                if let Some(end) = remaining.find('\n') {
                                    let mut hardware_type = remaining[..end].trim().to_string();
                                    if !hardware_type.contains("-") {
                                        if hardware_type == "Unknown" {
                                            log("当前板卡的类型为: Unknown, 弹窗判断");
                                            let result = show_dialog_and_wait(app_handle.clone(), "⚠️ 可能试屏幕接触不良导致无法判断版本，请手动选择：".to_string(), vec![
                                                serde_json::json!({ "text": "ATX" }),
                                                serde_json::json!({ "text": "Desk" })
                                            ]);
                                            hardware_type = result;
                                            auto_type = false;
                                            // 等待弹窗消失500ms
                                            std::thread::sleep(Duration::from_millis(500));
                                        }
                                        // 获取当前板卡的类型
                                        let hardware_version_str = get_config_str("testing", "board_version");
                                        // hardware_type = hardware_type-hardware_version_str
                                        hardware_type = format!("{}-{}", hardware_type, hardware_version_str.unwrap_or_default());
                                    }
                                    log(&format!("RUST检测到当前板卡的类型为: {}", hardware_type));
                                    set_current_hardware(app_handle.clone(), &hardware_type);
                                    target_name = format!("NanoKVM-{}", hardware_type);
                                    // NanoKVM_Pro (Desk-B) NeaR00293
                                    target_type = format!("NanoKVM_Pro ({}) ", hardware_type);
                                    if target_type.contains("ATX") {
                                        target_hardware_type = HardwareType::Atx;
                                    } else {
                                        target_hardware_type = HardwareType::Desk;
                                    }
                                }
                            }
                            // 获取或生成当前板卡的串号
                            if let Some(start) = output.find("当前板卡的串号为：") {
                                let content_start = start + "当前板卡的串号为：".len();
                                let remaining = &output[content_start..];
                                if let Some(end) = remaining.find('\n') {
                                    let serial_number = &remaining[..end].trim();
                                    log(&format!("RUST检测到当前板卡的串号为: {}", serial_number));
                                    set_target_serial(app_handle.clone(), serial_number);
                                    target_serial = serial_number.to_string();
                                }
                            } else {
                                // 生成新串号
                                let serial_number = create_serial_number(&target_name).unwrap_or_default();
                                log(&format!("生成新串号: {}", serial_number));
                                set_target_serial(app_handle.clone(), &serial_number);
                                target_serial = serial_number;
                            }
                            // 获取当前板卡是否有wifi模块
                            if output.contains("当前板卡有wifi模块") {
                                log("RUST检测到当前板卡有wifi模块");
                                wifi_exist = true;
                            }
                            // 获取soc id
                            if let Some(start) = output.find("SOC ID: ") {
                                let content_start = start + "SOC ID: ".len();
                                let remaining = &output[content_start..];
                                if let Some(end) = remaining.find('\n') {
                                    soc_id = remaining[..end].trim().to_string();
                                    log(&format!("RUST检测到当前板卡的soc id为: {}", soc_id));
                                }
                            }
                            let _ = set_test_status(&target_serial, "soc_uid", &soc_id);
                            let _ = set_test_status(&target_serial, "hardware", &target_type);
                            let _ = set_test_status(&target_serial, "wifi_exist", &wifi_exist.to_string());
                        }
                        Err(e) => {
                            log(&format!("SSH命令执行失败: {}", e));
                            set_step_status(app_handle.clone(), "detect_hardware", AppTestStatus::Repairing);
                            continue;
                        }
                    }

                    set_step_status(app_handle.clone(), "detect_hardware", AppTestStatus::Success);
                    app_step1_status = AppStepStatus::CheckingEmmc;  // 检查eMMC中
                }
                AppStepStatus::CheckingEmmc => {  // 检查eMMC中
                    log("检查eMMC中");
                    set_step_status(app_handle.clone(), "emmc_test", AppTestStatus::Testing);

                    match ssh_execute_command("/root/NanoKVM_Pro_Testing/test_sh/03_test_emmc.sh").await {
                        Ok(output) => {
                            if output.contains("eMMC test passed") {
                                log("eMMC测试通过");
                                set_step_status(app_handle.clone(), "emmc_test", AppTestStatus::Success);
                                app_step1_status = AppStepStatus::Printing;
                                let _ = set_test_status(&target_serial, "emmc", "Normal");
                                continue;
                            } else {
                                log("eMMC测试失败");
                                set_step_status(app_handle.clone(), "emmc_test", AppTestStatus::Failed);
                                // 弹窗是否再检测一遍
                                let response = show_dialog_and_wait(app_handle.clone(), "eMMC测试失败，是否再检测一遍".to_string(), vec![
                                    serde_json::json!({ "text": "再次检测" }),
                                    serde_json::json!({ "text": "否，直接打印不良" })
                                ]);
                                if response == "否，直接打印不良" {
                                    log("用户选择了直接打印不良");
                                    let _ = set_test_status(&target_serial, "emmc", "Damage");
                                    // 生成错误图片
                                    add_error_msg("eMMC异常，检查焊接 | ");

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
                                    app_step1_status = AppStepStatus::Finished;  // 跳转到结束
                                    continue;
                                } else {
                                    // 等待弹窗消失500ms
                                    std::thread::sleep(Duration::from_millis(500));
                                    log("用户选择了YES，重新检测eMMC");
                                    continue;
                                }
                            }
                        }
                        Err(e) => {
                            log(&format!("SSH命令执行失败: {}", e));
                        }
                    }
                }
                AppStepStatus::Printing => {  // 打印中
                    log("打印身份贴纸");
                    if is_printer_connected().await {
                        log("打印机已连接");
                        let img = generate_image_with_params(&target_serial, &target_name, wifi_exist);
                        if PRINTER_ENABLE {
                            if let Err(e) = print_image(&img, Some(TARGET_PRINTER)) {
                                log(&format!("打印图像失败: {}", e));
                                // #
                            }
                        }
                        app_step1_status = AppStepStatus::StartStep2;
                        continue;
                    } else {
                        log("打印机未连接,弹窗检测");
                        // 弹窗是否连接打印机
                        let _ = show_dialog_and_wait(app_handle.clone(), "⚠️ 打印机未连接或打印机驱动未安装，绿灯常亮可能是充电状态，长按侧边按钮开机".to_string(), vec![
                            serde_json::json!({ "text": "连接" }),
                        ]);
                        // 等待弹窗消失500ms
                        std::thread::sleep(Duration::from_millis(500));
                        continue;
                    }
                }
                AppStepStatus::StartStep2 => {  // 启动Step2
                    log("Step1完成，启动Step2内容");
                    let file_update_handle = spawn_step2_file_update(app_handle.clone());
                    let app_install_handle = spawn_step2_app_install(app_handle.clone());
                    let hdmi_testing_handle = spawn_step2_hdmi_testing(app_handle.clone(), &target_type, &target_serial);
                    let usb_testing_handle = spawn_step2_usb_testing(app_handle.clone(), &target_serial);
                    let eth_testing_handle = spawn_step2_eth_testing(app_handle.clone(), &target_serial, &current_static_ip);
                    // spawn_step2_eth_testing(app_handle.clone(), "192.168.1.7");
                    let wifi_testing_handle = spawn_step2_wifi_testing(app_handle.clone(), &target_serial, &ssid, &password, wifi_exist);
                    let penal_testing_handle = spawn_step2_penal_testing(app_handle.clone(), target_hardware_type.clone());
                    let ux_testing_handle = spawn_step2_ux_testing(app_handle.clone(), &target_serial, target_hardware_type.clone(), auto_type);
                    let atx_testing_handle = spawn_step2_atx_testing(app_handle.clone(), &target_serial, target_hardware_type.clone());
                    let io_testing_handle = spawn_step2_io_testing(app_handle.clone(), &target_serial);
                    let tf_testing_handle = spawn_step2_tf_testing(app_handle.clone(), &target_serial, target_hardware_type.clone());
                    let uart_testing_handle = spawn_step2_uart_testing(app_handle.clone(), &target_serial, target_hardware_type.clone());
                    log("Step2启动完成");

                    usb_testing_handle.await.unwrap();
                    log("USB测试完成");
                    eth_testing_handle.await.unwrap();
                    log("ETH测试完成");
                    wifi_testing_handle.await.unwrap();
                    log("WIFI测试完成");
                    penal_testing_handle.await.unwrap();
                    log("Penal测试完成");
                    ux_testing_handle.await.unwrap();
                    log("UX测试完成");
                    atx_testing_handle.await.unwrap();
                    log("ATX测试完成");
                    io_testing_handle.await.unwrap();
                    log("IO测试完成");
                    tf_testing_handle.await.unwrap();
                    log("TF测试完成");
                    uart_testing_handle.await.unwrap();
                    log("UART测试完成");
                    hdmi_testing_handle.await.unwrap();
                    log("HDMI测试完成");
                    file_update_handle.await.unwrap();
                    log("文件更新完成");
                    app_install_handle.await.unwrap();
                    log("应用安装完成");

                    log("Step2测试完成");
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    file_server_handle.abort();  // 下载完成后+网速测试完成后，终止文件服务器任务

                    app_step1_status = AppStepStatus::StartStep3;
                }
                AppStepStatus::StartStep3 => {  // 打印中
                    log("Step2测试完成，启动Step3内容");
                    let test_end_handle = spawn_step3_test_end(app_handle.clone(), &target_serial);
                    test_end_handle.await.unwrap();
                    app_step1_status = AppStepStatus::Finished;
                }
                AppStepStatus::Finished => {  // 完成
                    log("测试完成");
                    // 弹窗测试完成
                    std::thread::sleep(Duration::from_millis(500));
                    if all_step_status_is_success() {
                        
                        let _ = show_dialog_and_wait(app_handle.clone(), "测试完成，请拔出线缆".to_string(), vec![
                            serde_json::json!({ "text": "已拔出线缆" }),
                        ]);
                    } else {
                        let _ = show_dialog_and_wait(app_handle.clone(), "测试过程中出现错误，请粘贴错误贴纸，并拔出线缆".to_string(), vec![
                            serde_json::json!({ "text": "已拔出线缆" }),
                        ]);
                    }
                    std::thread::sleep(Duration::from_millis(500));
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    })
}
