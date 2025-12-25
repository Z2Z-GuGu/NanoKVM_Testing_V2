use std::time::Duration;
use tauri::async_runtime::spawn;
use tauri::{AppHandle};
use crate::threads::serial::{
    is_usb_tool_connected, get_current_data_density, 
    serial_send, detect_serial_string, execute_command_and_wait};
use crate::threads::dialog_test::{show_dialog_and_wait};
use crate::threads::update_state::{AppStep1Status, AppTestStatus, 
    set_step_status, clean_step1_status, set_target_ip, set_current_hardware, set_target_serial};
use crate::threads::server::spawn_file_server_task;
use crate::threads::ssh::ssh_execute_command;
use crate::threads::save::{get_config_str, create_serial_number};
use crate::threads::printer::{is_printer_connected, generate_image_with_params, print_image, PRINTER_ENABLE, TARGET_PRINTER};
use crate::threads::step2::{spawn_step2_file_update, spawn_step2_hdmi_testing, 
    spawn_step2_usb_testing, spawn_step2_net_testing};

const NOT_CONNECTED_KVM_COUNT_THRESHOLD: u64 = 10;  // 未连接KVM超过10次，同步弹窗提示,约10s

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
        let mut target_serial = String::new();
        let mut target_name = String::new();
        let mut target_type = String::new();
        let mut wifi_exist = false;
        let handle = spawn_file_server_task();     // 启动文件服务器任务
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
                    // 不连接测试主机时需要注释
                    // if ! execute_command_and_wait("sudo pkill dhclient\n", ":~#", 1000).await { 
                    //     log("关闭DHCP超时");
                    //     // ##
                    // }
                    // // 清空ip
                    // log("清空ip");
                    // if ! execute_command_and_wait("sudo ip addr flush dev eth0\n", ":~#", 1000).await {
                    //     log("清空ip超时");
                    //     // ##
                    // };
                    // // 设置静态IP
                    // log("设置静态IP");
                    // if ! execute_command_and_wait("sudo ip addr add 172.168.100.2/24 dev eth0\n", ":~#", 1000).await { 
                    //     log("设置静态IP超时");
                    //     // ##
                    // };
                    // while ! execute_command_and_wait("ping -c 1 172.168.100.1\n", "1 received", 1000).await {
                    //     set_step_status(app_handle.clone(), "get_ip", AppTestStatus::Repairing);
                    //     log("需要发送CTRL C确保退出ping");
                    //     if ! execute_command_and_wait("\x03", ":~#", 1000).await {
                    //         log("CTRL+C超时");
                    //         // ##
                    //     };
                    //     // 清空ip
                    //     log("清空ip");
                    //     if ! execute_command_and_wait("sudo ip addr flush dev eth0\n", ":~#", 1000).await {
                    //         log("清空ip超时");
                    //         // ##
                    //     };
                    //     // 设置静态IP
                    //     log("设置静态IP");
                    //     if ! execute_command_and_wait("sudo ip addr add 172.168.100.2/24 dev eth0\n", ":~#", 1000).await { 
                    //         log("设置静态IP超时");
                    //         // ##
                    //     };
                    // }
                    set_step_status(app_handle.clone(), "get_ip", AppTestStatus::Success);
                    set_target_ip(app_handle.clone(), "172.168.100.2");
                    app_step1_status = AppStep1Status::Download_File;  // 下载文件中
                }
                AppStep1Status::Download_File => {  // 下载文件中
                    log("下载文件中");
                    set_step_status(app_handle.clone(), "download_test", AppTestStatus::Testing);
                    // 检测文件是否存在：curl "http://192.168.2.201:8080/download" --output ./test.tar -s -o /dev/null -w "speed: %{speed_download} B/s\n"
                    if ! execute_command_and_wait("ls /root/test.tar\n", "cannot", 500).await {
                        let _ = execute_command_and_wait(" ", ":~#", 500).await;
                        // 找到了文件,弹窗是否重新下载
                        let response = show_dialog_and_wait(app_handle.clone(), "待测KVM中已存在产测文件，是否使用最新文件测试".to_string(), vec![
                            serde_json::json!({ "text": "YES" }),
                            serde_json::json!({ "text": "NO" })
                        ]);
                        if response == "NO" {
                            log("用户选择了不重新下载");
                            set_step_status(app_handle.clone(), "download_test", AppTestStatus::Success);
                            app_step1_status = AppStep1Status::Checking_Hardware;  // 未连接KVM
                            continue;
                        } else {
                            log("用户选择了重新下载");
                        }
                    } else {
                        let _ = execute_command_and_wait(" ", ":~#", 500).await;
                    }
                    
                    log("删除旧文件");
                    let _ = execute_command_and_wait("rm -rf /root/NanoKVM_Pro_Testing*\n", ":~#", 2000).await;
                    
                    log("开始下载");
                    std::thread::sleep(Duration::from_secs(2));                 // 等待文件服务器启动
                    // 检测文件是否存在：curl "http://172.168.100.2:8080/download" --output ./test.tar -s -o /dev/null -w "speed: %{speed_download} B/s\n"
                    let _ = execute_command_and_wait("curl \"http://192.168.1.7:8080/download\" --output /root/test.tar -s -o /dev/null \n", ":~#", 2000).await;
                    // let _ = execute_command_and_wait("curl \"http://192.168.2.201:8080/download\" --output /root/test.tar -s -o /dev/null \n", ":~#", 2000).await;
                    

                    log("找到文件,正在解压");
                    let _ = execute_command_and_wait("tar -xf /root/test.tar -C /root/\n", ":~#", 2000).await;

                    log("设置文件权限");
                    let _ = execute_command_and_wait("sudo chmod -R +x /root/NanoKVM_Pro_Testing*\n", ":~#", 2000).await;
                    
                    set_step_status(app_handle.clone(), "download_test", AppTestStatus::Success);
                    app_step1_status = AppStep1Status::Checking_Hardware;  // 检查硬件中
                }
                AppStep1Status::Checking_Hardware => {  // 检查硬件中
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
                        }
                        Err(e) => {
                            log(&format!("SSH命令执行失败: {}", e));
                        }
                    }

                    set_step_status(app_handle.clone(), "detect_hardware", AppTestStatus::Success);
                    app_step1_status = AppStep1Status::Checking_EMMC;  // 检查eMMC中
                }
                AppStep1Status::Checking_EMMC => {  // 检查eMMC中
                    log("检查eMMC中");
                    set_step_status(app_handle.clone(), "emmc_test", AppTestStatus::Testing);

                    match ssh_execute_command("/root/NanoKVM_Pro_Testing/test_sh/03_test_emmc.sh").await {
                        Ok(output) => {
                            if output.contains("eMMC test passed") {
                                log("eMMC测试通过");
                                set_step_status(app_handle.clone(), "emmc_test", AppTestStatus::Success);
                                app_step1_status = AppStep1Status::Printing;
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
                                    // ##
                                } else {
                                    log("用户选择了YES，重新检测eMMC");
                                    continue;
                                }
                                // 等待弹窗消失500ms
                                std::thread::sleep(Duration::from_millis(500));
                            }
                        }
                        Err(e) => {
                            log(&format!("SSH命令执行失败: {}", e));
                        }
                    }
                }
                AppStep1Status::Printing => {  // 打印中
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
                        app_step1_status = AppStep1Status::Finished;
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
                AppStep1Status::Finished => {  // 完成
                    log("Step1完成，启动Step2内容");
                    handle.abort();  // 下载完成后，终止文件服务器任务
                    spawn_step2_file_update(app_handle.clone());
                    spawn_step2_hdmi_testing(app_handle.clone(), &target_type, &target_serial);
                    spawn_step2_usb_testing(app_handle.clone());
                    spawn_step2_net_testing(app_handle.clone());
                    log("Step2启动完成");

                    loop {
                        log("sleep");
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    });
}
