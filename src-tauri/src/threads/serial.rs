use serialport::{SerialPortType};
use tokio_serial::{SerialStream, new, SerialPortBuilderExt};
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ErrorKind};
use tokio::time::{sleep, timeout};
use tauri::async_runtime::spawn;
use lazy_static::lazy_static;
use strip_ansi_escapes::strip;
use tauri::AppHandle;
use crate::threads::dialog_test::show_dialog_and_wait;

// 定义PID/VID和波特率
const TARGET_VID: u16 = 0x1a86;
const TARGET_PID: u16 = 0x55d3;
const SERIAL_BAUD_RATE: u32 = 115200;

const OVERTIME_ENTER: u64 = 3;          // 回车验活时间:3秒
const OVERTIME_LOST_KVM: u64 = 5;       // 最长无响应时间:5秒

// 登录默认账号密码
const DEFAULT_USERNAME: &str = "root";
const DEFAULT_PASSWORD: &str = "sipeed";


const MAX_UPDATE_CONNECT_ERR_COUNT: u32 = 2;        // 最大连接错误次数，超过则认为连接失败
const MAX_CONNECT_DETECTE_DUTY_MS: u64 = 500;       // 检测周期500ms
const MAX_RECEIVE_DATA_TIMEOUT_MS: u64 = 100;       // 最大接收数据超时时间100ms
const FILTER_WINDOW_SIZE: usize = 5;                // 滑动滤波器窗口大小（时间=5*MAX_RECEIVE_DATA_TIMEOUT_MS）

// 超时时间
const OVERTIME_LOGIN: u64 = 20000;      // 登录超时时间:60秒
const OVERTIME_PASSWORD: u64 = 5000;    // 密码超时时间:5秒
const OVERTIME_LOGIN_SUCCESS: u64 = 10000;    // 登录成功超时时间:10秒
const OVERTIME_COMMAND: u64 = 2000;    // 普通命令超时时间:2秒
const OVERTIME_COMMAND_SHORT: u64 = 1000;    // 短命令超时时间:1秒
const OVERTIME_CONNECTED: u64 = 10000;    // 最长容忍的Connected时间：10s
const OVERTIME_ENTERED: u64 = 3000;    // 检测不到后发送回车时间：3s


const DISCONNECT_COUNT_THRESHOLD: u32 = 4; // 检测不到USB工具3次后触发弹窗

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = true;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[serial]{}", msg);
    }
}

// 定义全局收发队列和USB工具状态
lazy_static! {
    pub static ref SEND_QUEUE: Mutex<Option<mpsc::Sender<Vec<u8>>>> = Mutex::new(None);         // 串口数据发送队列
    pub static ref RECEIVE_QUEUE: Mutex<Option<mpsc::Receiver<Vec<u8>>>> = Mutex::new(None);    // 串口数据接收队列
    pub static ref USB_TOOL_CONNECTED: Mutex<bool> = Mutex::new(false);                         // USB工具状态全局变量
    static ref WINDOW: Mutex<Vec<u32>> = Mutex::new(Vec::with_capacity(FILTER_WINDOW_SIZE));    // 滑动滤波器窗口
    pub static ref DATA_DENSITY: Mutex<u32> = Mutex::new(0);                                    // 数据密度全局变量
}

// 滑动滤波器函数，数组位于函数内部，每输入一个数字，就计算当前窗口的总数值，并删除最早的一个
async fn slide_filter(new_value: u32) -> u32 {
    let mut window = WINDOW.lock().await;
    
    window.push(new_value);
    if window.len() > FILTER_WINDOW_SIZE {
        window.remove(0);
    }
    window.iter().sum()
}

// 清空滑动滤波器窗口
async fn clear_slide_filter() {
    let mut window = WINDOW.lock().await;
    window.clear();
}

// 扫描指定PID/VID的串口
fn scan_serial_port(target_pid: u16, target_vid: u16) -> Option<String> {
    if let Ok(ports) = serialport::available_ports() {
        for port_info in ports {
            if let SerialPortType::UsbPort(usb_info) = port_info.port_type {
                if usb_info.vid == target_vid && usb_info.pid == target_pid {
                    return Some(port_info.port_name);
                }
            }
        }
    }
    None
}

// 连接指定（PID/VID）串口函数
async fn connect_serial_port(port_name: &str) -> Option<Arc<Mutex<SerialStream>>> {
    let port = new(port_name, SERIAL_BAUD_RATE);
    
    match port.open_native_async() {
        Ok(serial_port) => Some(Arc::new(Mutex::new(serial_port))),
        Err(e) => {
            log(&format!("打开串口错误: {:?}", e));
            None
        }
    }
}

// 串口发送函数（向写队列写）
pub async fn serial_send(data: &str) {
    let send_data = data.as_bytes().to_vec();
    
    // 获取全局发送队列
    let send_queue_guard = SEND_QUEUE.lock().await;
    if let Some(send_queue) = &*send_queue_guard {
        if let Err(e) = send_queue.send(send_data).await {
            log(&format!("发送数据到发送队列失败: {:?}", e));
        }
    } else {
        log("发送队列未初始化");
    }
}

// 串口接收函数（从读队列读）
pub async fn serial_receive() -> String {
    // 获取全局接收队列
    let mut receive_queue_guard = RECEIVE_QUEUE.lock().await;
    if let Some(receive_queue) = &mut *receive_queue_guard {
        match receive_queue.try_recv() {
            Ok(data) => String::from_utf8_lossy(&data).to_string(),
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => String::new(),
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => String::from("接收队列已关闭"),
        }
    } else {
        String::from("接收队列未初始化")
    }
}

// 从队列获取数据并清理 ANSI 转义序列
pub async fn serial_receive_clean() -> String {
    // 先获取原始数据
    let raw_data = serial_receive().await;
    
    // 如果数据为空或包含错误信息，直接返回
    if raw_data.is_empty() || 
       raw_data == "接收队列已关闭" || 
       raw_data == "接收队列未初始化" {
        return raw_data;
    }
    
    // 清理 ANSI 转义序列
    let cleaned_bytes = strip(raw_data.as_bytes());
    match String::from_utf8(cleaned_bytes) {
        Ok(cleaned_string) => {
            // 去除首尾空白字符
            cleaned_string.trim().to_string()
        }
        Err(_) => raw_data, // 如果 UTF-8 转换失败，返回原始数据
    }
}

// 串口管理线程函数
pub fn serial_management_task() {
    spawn(async move {
        let mut serial_port: Option<Arc<Mutex<SerialStream>>> = None;
        let mut serial_connect_err_count = 0;
        clear_slide_filter();
        
        // 创建发送和接收队列
        let (send_queue_tx, mut send_queue_rx) = mpsc::channel::<Vec<u8>>(100);
        let (receive_queue_tx, receive_queue_rx) = mpsc::channel::<Vec<u8>>(100);
        
        // 将队列存储到全局静态变量中
        {
            let mut send_queue_guard = SEND_QUEUE.lock().await;
            *send_queue_guard = Some(send_queue_tx);
        }
        
        {
            let mut receive_queue_guard = RECEIVE_QUEUE.lock().await;
            *receive_queue_guard = Some(receive_queue_rx);
        }

        
        log("串口管理线程已启动");
        
        loop {
            // 如果没有连接就开始尝试连接
            if serial_port.is_none() {
                log("串口连接错误，尝试连接...");
                // 尝试连接
                if let Some(port_name) = scan_serial_port(TARGET_PID, TARGET_VID) {
                    if let Some(port) = connect_serial_port(&port_name).await {
                        // log("成功连接串口");
                        *USB_TOOL_CONNECTED.lock().await = true;
                        serial_port = Some(port);
                        serial_connect_err_count = 0;
                    }
                }
                
                // 如果还是没有
                if serial_port.is_none() {
                    // 没有PIDVID设备连接
                    clear_slide_filter().await;
                    serial_connect_err_count += 1;
                    if serial_connect_err_count >= MAX_UPDATE_CONNECT_ERR_COUNT {
                        log("串口连接多次失败，标记为未连接");
                        *USB_TOOL_CONNECTED.lock().await = false;
                        serial_connect_err_count = MAX_UPDATE_CONNECT_ERR_COUNT;
                    }
                    sleep(Duration::from_millis(MAX_CONNECT_DETECTE_DUTY_MS)).await;
                    continue;
                }
            }
            
            // 如果连接了就交替读写串口到队列
            let mut disconnect_needed = false;
            
            // 1. 尝试从发送队列读取数据并发送到串口
            match send_queue_rx.try_recv() {
                Ok(data) => {
                    // log(&format!("发送数据: {:?}", data));
                    if let Ok(mut port_guard) = serial_port.as_ref().unwrap().try_lock() {
                        if let Err(e) = port_guard.write_all(&data).await {
                            log(&format!("发送数据失败: {:?}", e));
                            // 当遇到管道断开或权限被拒绝错误时，断开连接
                            if e.kind() == ErrorKind::BrokenPipe || e.kind() == ErrorKind::PermissionDenied {
                                disconnect_needed = true;
                            }
                        }
                    }
                }
                Err(_) => {
                    // 发送队列为空，继续下一步
                }
            }
            
            // 2. 尝试从串口读取数据并发送到接收队列
            if let Ok(mut port_guard) = serial_port.as_ref().unwrap().try_lock() {
                let mut buffer = vec![0; 1024];
                match timeout(Duration::from_millis(MAX_RECEIVE_DATA_TIMEOUT_MS), port_guard.read(&mut buffer)).await {
                    Ok(Ok(bytes_read)) => {
                        // 计算当前数据密度
                        *DATA_DENSITY.lock().await = slide_filter(bytes_read.try_into().unwrap()).await;

                        if bytes_read > 0 {
                            let received_data = buffer[..bytes_read].to_vec();
                            // log(&format!("接收数据: {:?}", received_data));
                            // if let Err(e) = receive_queue_tx.send(received_data).await {
                            //     log(&format!("发送数据到接收队列失败: {:?}", e));
                            // }
                            match receive_queue_tx.try_send(received_data) {
                                Ok(_) => {},
                                Err(crate::threads::serial::mpsc::error::TrySendError::Full(data)) => {
                                    // log("队列已满，丢弃最早数据");
                                    let _ = serial_receive().await;
                                    // 重试发送当前数据
                                    let _ = receive_queue_tx.try_send(data);
                                },
                                Err(_) => {},
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        // log(&format!("读取数据失败: {:?}", e));
                        // 当遇到管道断开或权限被拒绝错误时，断开连接
                        if e.kind() == ErrorKind::BrokenPipe || e.kind() == ErrorKind::PermissionDenied {
                            disconnect_needed = true;
                        }
                    }
                    Err(_) => {
                        // 计算当前数据密度
                        *DATA_DENSITY.lock().await = slide_filter(0).await;
                    }
                }
            }

            log(&format!("当前数据密度: {:?}", *DATA_DENSITY.lock().await));
            
            // 如果需要断开连接
            if disconnect_needed {
                log("断开串口连接");
                serial_port = None;
            }
            
            // 短暂休眠，避免CPU占用过高
            sleep(Duration::from_millis(10)).await;
        }
    });
}

// 等待指定串口数据，带超时设置，单位：毫秒
async fn wait_for_serial_data(expected: &[u8], timeout_ms: u64) -> bool {
    let timeout_time = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        let received = serial_receive_clean().await;
        if !received.is_empty() {
            if received.as_bytes().windows(expected.len()).any(|window| window == expected) {
                // log(&format!("当前串口数据: {:?}", received));
                return true;    
            }
        }
        // log(&format!("等待串口数据{:?}当前超时剩余时间: {:?}", expected, timeout_time - Instant::now()));
        if Instant::now() >= timeout_time {
            return false;
        }

        // 短暂休眠，避免忙等待，使用最长的队列填充时间
        sleep(Duration::from_millis(MAX_RECEIVE_DATA_TIMEOUT_MS)).await;
    }
}

// // 修改USB工具状态为Connected
// async fn set_usb_tool_status_connected() {
//     // 如果是ConnectKVM状态才会修改到Connected
//     let current_status = USB_TOOL_CONNECTED.lock().await;
//     if *current_status != UsbToolStatus::ConnectKVM {
//         drop(current_status); // 提前释放锁
//         log("非连接KVM状态，直接退出");
//         return;
//     }
//     drop(current_status); // 提前释放锁

//     let mut usb_tool_status = USB_TOOL_CONNECTED.lock().await;
//     *usb_tool_status = UsbToolStatus::Connected;
// }

// 执行命令并等待完成
async fn execute_command_and_wait(command: &str, ret_str: &str, exit: bool) -> bool {
    serial_send(command).await;
    if ! wait_for_serial_data(ret_str.as_bytes(), OVERTIME_COMMAND).await {
        if exit {
            log(&format!("发送{}超时", command));
            // set_usb_tool_status_connected().await;
        }
        return false;
    }
    return true;
}

// 串口系统操作线程
// async fn serial_system_operation_task() {
//     // 如果非连接KVM状态直接退出
//     let current_status = USB_TOOL_CONNECTED.lock().await;
//     if *current_status != UsbToolStatus::ConnectKVM {
//         drop(current_status); // 提前释放锁
//         log("非连接KVM状态，直接退出");
//         return;
//     }
//     drop(current_status); // 提前释放锁

//     // delsy 10s 避免进入uboot
//     log("正在启动...");
//     sleep(Duration::from_secs(10)).await;

//     // 发送回车看有没有#
//     serial_send("\r\n").await;
//     if ! wait_for_serial_data(b":~#", OVERTIME_COMMAND_SHORT).await {
//         // 未登录状态需要登录
//         serial_send("\r\n").await;
//         // 等待登录提示
//         log("等待登录提示");
//         if ! wait_for_serial_data(b"login:", OVERTIME_LOGIN).await {
//             log("登录超时");
//             set_usb_tool_status_connected().await;
//             return;
//         }
//         // 发送用户名
//         log("发送用户名");
//         let username = format!("{}\r\n", DEFAULT_USERNAME);
//         serial_send(&username).await;
//         // 等待密码提示
//         if ! wait_for_serial_data(b"Password:", OVERTIME_PASSWORD).await {
//             log("密码超时");
//             set_usb_tool_status_connected().await;
//             return;
//         }
//         // 发送密码
//         log("发送密码");
//         let password = format!("{}\r\n", DEFAULT_PASSWORD);
//         serial_send(&password).await;
//         // 等待登录成功
//         if ! wait_for_serial_data(b"#", OVERTIME_LOGIN_SUCCESS).await {
//             log("登录错误");
//             set_usb_tool_status_connected().await;
//             return;
//         }
//     }

//     // 设置静态IP
//     // 关闭dhcp
//     log("关闭dhcp");
//     if ! execute_command_and_wait("sudo pkill dhclient \r\n", "#", true).await { return };
//     // 验证IP是否设置成功
//     log("验证IP是否设置成功");
//     while ! execute_command_and_wait("ping -c 1 172.168.100.1\r\n", "time", false).await {
//         // 清空ip
//         log("清空ip");
//         if ! execute_command_and_wait("sudo ip addr flush dev eth0\r\n", "#", true).await { return };
//         // 设置静态IP
//         log("设置静态IP");
//         if ! execute_command_and_wait("sudo ip addr add 172.168.100.2/24 dev eth0\r\n", "#", true).await { return };
//     }

//     loop {
//         // sleep 1s
//         sleep(Duration::from_secs(1)).await;
//         log("sleep");

//         // 检查USB工具状态是否为ConnectKVM
//         let current_status = USB_TOOL_CONNECTED.lock().await;
//         if *current_status != UsbToolStatus::ConnectKVM {
//             drop(current_status); // 提前释放锁
//             log("非连接KVM状态，直接退出");
//             return;
//         }
//         drop(current_status); // 提前释放锁

//         // 回车验活
//         if ! execute_command_and_wait("\n", "#", true).await { return };
//     }
// }

// // 串口数据管理线程
pub fn serial_data_management_task(_app_handle: AppHandle) {
    spawn(async move {
        loop {
            // if *USB_TOOL_CONNECTED.lock().await {
            //     // 打印接收数据
            //     let received_data = serial_receive_clean().await;
            //     // if *DATA_DENSITY.lock().await > 0 {
            //     if !received_data.is_empty() {
            //         log(&format!("接收数据: {:?}", received_data));
            //     }
            // }
            // 短暂休眠，避免CPU占用过高
            sleep(Duration::from_millis(10)).await;
        }
    });
}

// 检测USB工具是否已经连接
pub async fn is_usb_tool_connected() -> bool {
    *USB_TOOL_CONNECTED.lock().await
}
