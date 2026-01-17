use serialport::{SerialPortType};
use tokio_serial::{SerialStream, new, SerialPortBuilderExt};
use tokio_serial::SerialPort; // 引入特质以使用 DTR/RTS 方法
use std::time::{Duration, Instant};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tokio::sync::{Mutex, mpsc};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ErrorKind};
use tokio::time::{sleep, timeout};
use tauri::async_runtime::spawn;
use lazy_static::lazy_static;
use strip_ansi_escapes::strip;
use crate::APP_EXIT;


// 定义PID/VID和波特率
const TARGET_VID: u16 = 0x1a86;
const TARGET_PID: u16 = 0x55d3;
const SERIAL_BAUD_RATE: u32 = 115200;

// const OVERTIME_ENTER: u64 = 3;          // 回车验活时间:3秒
// const OVERTIME_LOST_KVM: u64 = 5;       // 最长无响应时间:5秒

// 登录默认账号密码
// const DEFAULT_USERNAME: &str = "root";
// const DEFAULT_PASSWORD: &str = "sipeed";


const MAX_UPDATE_CONNECT_ERR_COUNT: u32 = 20;        // 最大连接错误次数，超过则认为连接失败
const MAX_CONNECT_DETECTE_DUTY_MS: u64 = 500;       // 检测周期500ms
const MAX_RECEIVE_DATA_TIMEOUT_MS: u64 = 100;       // 最大接收数据超时时间100ms
const FILTER_WINDOW_SIZE: usize = 10;                // 滑动滤波器窗口大小（时间=5*MAX_RECEIVE_DATA_TIMEOUT_MS）

// 超时时间
// const OVERTIME_LOGIN: u64 = 20000;      // 登录超时时间:60秒
// const OVERTIME_PASSWORD: u64 = 5000;    // 密码超时时间:5秒
// const OVERTIME_LOGIN_SUCCESS: u64 = 10000;    // 登录成功超时时间:10秒
// const OVERTIME_COMMAND: u64 = 2000;    // 普通命令超时时间:2秒
// const OVERTIME_COMMAND_SHORT: u64 = 1000;    // 短命令超时时间:1秒
// const OVERTIME_CONNECTED: u64 = 10000;    // 最长容忍的Connected时间：10s
// const OVERTIME_ENTERED: u64 = 3000;    // 检测不到后发送回车时间：3s


// const DISCONNECT_COUNT_THRESHOLD: u32 = 4; // 检测不到USB工具3次后触发弹窗

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
    pub static ref USB_TOOL_CONNECTED: AtomicBool = AtomicBool::new(false);                         // USB工具状态全局变量
    static ref WINDOW: Mutex<Vec<u32>> = Mutex::new(Vec::with_capacity(FILTER_WINDOW_SIZE));    // 滑动滤波器窗口
    pub static ref DATA_DENSITY: AtomicU32 = AtomicU32::new(0);                                    // 数据密度全局变量
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
    // 获取全局接收队列的可变引用，不移除队列
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
        let mut io_value: bool = true;
        clear_slide_filter().await;
        
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
   // 主循环
        loop {
            // 检查退出标志
            if APP_EXIT.load(Ordering::Relaxed) {
                serial_port = None;
                log(&format!("串口线程已退出, {:?}", serial_port.is_none()));
                break;
            }
            
            // 如果没有连接就开始尝试连接
            if serial_port.is_none() {
                log("串口连接错误，尝试连接...");
                // 尝试连接
                if let Some(port_name) = scan_serial_port(TARGET_PID, TARGET_VID) {
                    if let Some(port) = connect_serial_port(&port_name).await {
                        log("成功连接串口");
                        USB_TOOL_CONNECTED.store(true, Ordering::Relaxed);
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
                        USB_TOOL_CONNECTED.store(false, Ordering::Relaxed);
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
                        let density = slide_filter(bytes_read.try_into().unwrap()).await;
                        DATA_DENSITY.store(density, Ordering::Relaxed);

                        if bytes_read > 0 {
                            let received_data = buffer[..bytes_read].to_vec();
                            // log(&format!("接收数据: {:?}", received_data));
                            // if let Err(e) = receive_queue_tx.send(received_data).await {
                            //     log(&format!("发送数据到接收队列失败: {:?}", e));
                            // }
                            match receive_queue_tx.try_send(received_data) {
                                Ok(_) => {},
                                Err(crate::function::serial::mpsc::error::TrySendError::Full(data)) => {
                                    log("队列已满，丢弃最早数据");
                                    _ = serial_receive().await;
                                    // 直接重试发送当前数据
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
                        let density = slide_filter(0).await;
                        DATA_DENSITY.store(density, Ordering::Relaxed);
                    }
                }
            }

            // ctrl dtr&rts io
            if let Ok(mut port_guard) = serial_port.as_ref().unwrap().try_lock() {
                let _ = port_guard.write_data_terminal_ready(io_value);
                let _ = port_guard.write_request_to_send(io_value);
                io_value = !io_value;
            }

            // log(&format!("当前数据密度: {:?}", *DATA_DENSITY.lock().await));
            
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
#[allow(dead_code)]
pub async fn wait_for_serial_data(expected: &[u8], timeout_ms: u64) -> bool {
    let timeout_time = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        // 检查退出标志
        if APP_EXIT.load(Ordering::Relaxed) {
            log("程序退出，中断wait_for_serial_data循环");
            return false;
        }
        
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
        // sleep(Duration::from_millis(MAX_RECEIVE_DATA_TIMEOUT_MS)).await;
    }
}

// 执行命令并等待完成
pub async fn execute_command_and_wait(command: &str, ret_str: &str, timeout_ms: u64) -> bool {
    serial_send(command).await;
    let result = detect_serial_string(&[ret_str], timeout_ms, 0).await;
    log(&format!("detect_serial_string result: {:?}", result));
    
    result == ret_str
}

// 检测接收到的数据中是否包含一个列表中的字符串，如果包含回复匹配的字符串，不包含回复”UNMATCHED“，如果无数据回复”NO-DATA“，如果过程中数据密度过低则返回”LOW-DENSITY“
// 对于这个函数遇到一个问题：登录期间较长比如给定的30s，如果期间断开KVM连接（无数据/数据密度过低），或断开工具连接需要直接返回一定的数值，而不是硬等（可能需要数据密度做判别了）
pub async fn detect_serial_string(patterns: &[&str], timeout_ms: u64, min_density: u32) -> String {
    // log(&format!("detect_serial_string timeout_ms: {}", timeout_ms));
    let timeout_time = Instant::now() + Duration::from_millis(timeout_ms);
    let mut has_data = false;
    
    loop {
        // 检查退出标志
        if APP_EXIT.load(Ordering::Relaxed) {
            log("程序退出，中断detect_serial_string循环");
            return "UNMATCHED".to_string();
        }
        
        // log("判断数据密度");
        if min_density != 0 {
            let density = get_current_data_density().await;
            if density < min_density {
                // log(&format!("当前数据密度: {:?}", density));
                return "LOW-DENSITY".to_string();
            }
        }
        // log("等待串口数据");
        let received = serial_receive_clean().await;
        if !received.is_empty() {
            has_data = true;

            log(&format!("当前串口数据: {:?}", received));
            
            // 检查所有匹配模式
            for pattern in patterns {
                if received.contains(pattern) {
                    return pattern.to_string();
                }
            }
        }
        
        // 检查超时
        // log(&format!("等待串口数据超时剩余时间: {:?}", timeout_time - Instant::now()));
        if Instant::now() >= timeout_time {
            if !has_data {
                return "NO-DATA".to_string();
            } else {
                return "UNMATCHED".to_string();
            }
        }
        
        // 使用同步休眠，避免异步调度问题
        std::thread::sleep(Duration::from_millis(10));
    }
}

// pub async fn ensure_serial_terminal() -> bool {
//     log("ensure_serial_terminal");
//     clear_receive_queue().await;
//     // 确保串口终端可以写入命令：发送一个换行符，等待回复#
//     if !execute_command_and_wait("\n", "#", 1000).await {
//         // 如果不行发送CtrlC
//         clear_receive_queue().await;
//         if !execute_command_and_wait("\x03", "#", 1000).await {
//             // 直接返回异常
//             return false;
//         }
//     }
//     clear_receive_queue().await;
//     true
// }


// 检测USB工具是否已经连接
pub async fn is_usb_tool_connected() -> bool {
    log("is_usb_tool_connected");
    USB_TOOL_CONNECTED.load(Ordering::Relaxed)
}

// 获取当前串口数据密度
pub async fn get_current_data_density() -> u32 {
    DATA_DENSITY.load(Ordering::Relaxed)
}
