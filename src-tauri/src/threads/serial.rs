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
use crate::threads::timer::{TimerHandle, create_timer, check_timer, reset_timer, delete_timer};

// 定义PID/VID和波特率
const TARGET_VID: u16 = 0x1a86;
const TARGET_PID: u16 = 0x55d3;
const SERIAL_BAUD_RATE: u32 = 115200;

const OVERTIME_ENTER: u64 = 3;          // 回车验活时间:3秒
const OVERTIME_LOST_KVM: u64 = 5;       // 最长无响应时间:5秒

// 登录默认账号密码
const DEFAULT_USERNAME: &str = "root";
const DEFAULT_PASSWORD: &str = "sipeed";

// 超时时间
const OVERTIME_LOGIN: u64 = 60000;      // 登录超时时间:60秒
const OVERTIME_PASSWORD: u64 = 5000;    // 密码超时时间:5秒
const OVERTIME_LOGIN_SUCCESS: u64 = 10000;    // 登录成功超时时间:10秒
const OVERTIME_COMMAND: u64 = 2000;    // 普通命令超时时间:2秒
const OVERTIME_COMMAND_SHORT: u64 = 1000;    // 短命令超时时间:1秒



// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = false;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[serial]{}", msg);
    }
}

// 枚举USB工具状态队列状态
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum UsbToolStatus {
    Unknown,        // 未知状态/初始状态
    Connected,      // 瞬态：USB工具已连接（未连接NanoKVM-Pro）
    ConnectKVM,     // 瞬态：已连接到NanoKVM-Pro
    Disconnected,   // 瞬态：USB工具已断开
}

// 定义全局收发队列和USB工具状态
lazy_static! {
    pub static ref SEND_QUEUE: Mutex<Option<mpsc::Sender<Vec<u8>>>> = Mutex::new(None);         // 串口数据发送队列
    pub static ref RECEIVE_QUEUE: Mutex<Option<mpsc::Receiver<Vec<u8>>>> = Mutex::new(None);    // 串口数据接收队列
    pub static ref USB_TOOL_STATUS: Mutex<UsbToolStatus> = Mutex::new(UsbToolStatus::Unknown);  // USB工具状态全局变量
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
async fn serial_management_task() {
    let mut serial_port: Option<Arc<Mutex<SerialStream>>> = None;
    let mut last_connection_status = UsbToolStatus::Unknown;
    let mut serial_connect_err_count = 0;
    const MAX_SERIAL_CONNECT_ERR_COUNT: u32 = 100;
    const RECONNECT_MIN_SERIAL_CONNECT_ERR_COUNT: u32 = 2;
    const RECONNECT_MAX_SERIAL_CONNECT_ERR_COUNT: u32 = 8;
    
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
        // 如果没有连接就连接指定（PID/VID）串口
        if serial_port.is_none() {
            // log("检查串口连接状态...");
            if let Some(port_name) = scan_serial_port(TARGET_PID, TARGET_VID) {
                if last_connection_status == UsbToolStatus::Unknown || last_connection_status == UsbToolStatus::Disconnected {
                    // log("✅USB测试器已连接");
                    last_connection_status = UsbToolStatus::Connected;
                    // 更新全局USB工具状态
                    let mut status_guard = USB_TOOL_STATUS.lock().await;
                    *status_guard = last_connection_status;
                }
                // log(&format!("找到串口: {}", port_name));
                if let Some(port) = connect_serial_port(&port_name).await {
                    // log("成功连接串口");
                    serial_port = Some(port);
                    if serial_connect_err_count >= RECONNECT_MIN_SERIAL_CONNECT_ERR_COUNT {
                        if serial_connect_err_count <= RECONNECT_MAX_SERIAL_CONNECT_ERR_COUNT {
                            // log(&format!("检测到NanoKVM-Pro连接"));
                            last_connection_status = UsbToolStatus::ConnectKVM;
                        } else {
                            // log("理论不存在的情况：串口连接错误次数超过最大重试次数，暂时认为连接到KVM");
                            last_connection_status = UsbToolStatus::ConnectKVM;
                        }
                        // 更新全局USB工具状态
                        let mut status_guard = USB_TOOL_STATUS.lock().await;
                        *status_guard = last_connection_status;
                    } else {
                        // log("USB测试工具已连接，但未检测到NanoKVM-Pro");
                        // 更新全局USB工具状态
                        let mut status_guard = USB_TOOL_STATUS.lock().await;
                        *status_guard = last_connection_status;
                    }
                    serial_connect_err_count = 0;
                } else {
                    // 找到了，但是连接失败
                    serial_connect_err_count += 1;
                    if serial_connect_err_count >= MAX_SERIAL_CONNECT_ERR_COUNT {
                        serial_connect_err_count = MAX_SERIAL_CONNECT_ERR_COUNT;
                    }
                    // log(&format!("连接串口失败{}次", serial_connect_err_count));
                }
            } else {
                // 找不到USB测试器
                if last_connection_status != UsbToolStatus::Disconnected {
                    // log("❌USB测试器已断开");
                    last_connection_status = UsbToolStatus::Disconnected;
                    // 更新全局USB工具状态
                    let mut status_guard = USB_TOOL_STATUS.lock().await;
                    *status_guard = last_connection_status;
                } else {
                    serial_connect_err_count = 0;
                }
            }
            sleep(Duration::from_secs(1)).await;
            continue;
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
            match timeout(Duration::from_millis(100), port_guard.read(&mut buffer)).await {
                Ok(Ok(bytes_read)) => {
                    if bytes_read > 0 {
                        let received_data = buffer[..bytes_read].to_vec();
                        // log(&format!("接收数据: {:?}", received_data));
                        if let Err(e) = receive_queue_tx.send(received_data).await {
                            log(&format!("发送数据到接收队列失败: {:?}", e));
                        }
                    }
                }
                Ok(Err(e)) => {
                    log(&format!("读取数据失败: {:?}", e));
                    // 当遇到管道断开或权限被拒绝错误时，断开连接
                    if e.kind() == ErrorKind::BrokenPipe || e.kind() == ErrorKind::PermissionDenied {
                        disconnect_needed = true;
                    }
                }
                Err(_) => {
                    // 读取超时，这是正常的，继续循环
                }
            }
        }
        
        // 如果需要断开连接
        if disconnect_needed {
            log("断开串口连接");
            serial_port = None;
        }
        
        // 短暂休眠，避免CPU占用过高
        sleep(Duration::from_millis(10)).await;
    }
}

// 等待指定串口数据，带超时设置，单位：毫秒
async fn wait_for_serial_data(expected: &[u8], timeout_ms: u64) -> bool {
    // log(&format!("等待串口数据: {:?}", expected));
    // let mut received = String::new();
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
    }
}

// 修改USB工具状态为Connected
async fn set_usb_tool_status_connected() {
    // 如果是ConnectKVM状态才会修改到Connected
    let current_status = USB_TOOL_STATUS.lock().await;
    if *current_status != UsbToolStatus::ConnectKVM {
        drop(current_status); // 提前释放锁
        log("非连接KVM状态，直接退出");
        return;
    }
    drop(current_status); // 提前释放锁

    let mut usb_tool_status = USB_TOOL_STATUS.lock().await;
    *usb_tool_status = UsbToolStatus::Connected;
}

// 执行命令并等待完成
async fn execute_command_and_wait(command: &str, ret_str: &str, exit: bool) -> bool {
    serial_send(command).await;
    if ! wait_for_serial_data(ret_str.as_bytes(), OVERTIME_COMMAND).await {
        if exit {
            log(&format!("发送{}超时", command));
            set_usb_tool_status_connected().await;
        }
        return false;
    }
    return true;
}

// 串口系统操作线程
async fn serial_system_operation_task() {
    // 如果非连接KVM状态直接退出
    let current_status = USB_TOOL_STATUS.lock().await;
    if *current_status != UsbToolStatus::ConnectKVM {
        drop(current_status); // 提前释放锁
        log("非连接KVM状态，直接退出");
        return;
    }
    drop(current_status); // 提前释放锁

    // delsy 10s 避免进入uboot
    log("正在启动...");
    sleep(Duration::from_secs(10)).await;

    // 发送回车看有没有#
    serial_send("\r\n").await;
    if ! wait_for_serial_data(b":~#", OVERTIME_COMMAND_SHORT).await {
        // 未登录状态需要登录
        serial_send("\r\n").await;
        // 等待登录提示
        log("等待登录提示");
        if ! wait_for_serial_data(b"login:", OVERTIME_LOGIN).await {
            log("登录超时");
            set_usb_tool_status_connected().await;
            return;
        }
        // 发送用户名
        log("发送用户名");
        let username = format!("{}\r\n", DEFAULT_USERNAME);
        serial_send(&username).await;
        // 等待密码提示
        if ! wait_for_serial_data(b"Password:", OVERTIME_PASSWORD).await {
            log("密码超时");
            set_usb_tool_status_connected().await;
            return;
        }
        // 发送密码
        log("发送密码");
        let password = format!("{}\r\n", DEFAULT_PASSWORD);
        serial_send(&password).await;
        // 等待登录成功
        if ! wait_for_serial_data(b"#", OVERTIME_LOGIN_SUCCESS).await {
            log("登录错误");
            set_usb_tool_status_connected().await;
            return;
        }
    }

    // 设置静态IP
    // 关闭dhcp
    log("关闭dhcp");
    if ! execute_command_and_wait("sudo pkill dhclient \r\n", "#", true).await { return };
    // 验证IP是否设置成功
    log("验证IP是否设置成功");
    while ! execute_command_and_wait("ping -c 1 172.168.100.1\r\n", "time", false).await {
        // 清空ip
        log("清空ip");
        if ! execute_command_and_wait("sudo ip addr flush dev eth0\r\n", "#", true).await { return };
        // 设置静态IP
        log("设置静态IP");
        if ! execute_command_and_wait("sudo ip addr add 172.168.100.2/24 dev eth0\r\n", "#", true).await { return };
    }

    loop {
        // sleep 1s
        sleep(Duration::from_secs(1)).await;
        log("sleep");

        // 检查USB工具状态是否为ConnectKVM
        let current_status = USB_TOOL_STATUS.lock().await;
        if *current_status != UsbToolStatus::ConnectKVM {
            drop(current_status); // 提前释放锁
            log("非连接KVM状态，直接退出");
            return;
        }
        drop(current_status); // 提前释放锁

        // 回车验活
        if ! execute_command_and_wait("\n", "#", true).await { return };
    }
}

// 串口数据管理线程
async fn serial_data_management_task() {
    let mut last_connection_status = UsbToolStatus::Unknown;
    let mut none_resv_timer_handle : Option<TimerHandle> = None;

    loop {
        // 从全局变量获取USB工具状态
        let current_status = USB_TOOL_STATUS.lock().await;
        let status = *current_status;
        drop(current_status); // 提前释放锁

        // 如果状态有变化，更新本地状态
        if status != last_connection_status {
            log(&format!("USB工具状态变化: {:?}", status));
            last_connection_status = status;
            if status == UsbToolStatus::Connected || status == UsbToolStatus::ConnectKVM {
                // 一旦进入连接状态就开始计时
                none_resv_timer_handle = Some(create_timer(100).await);
                if status == UsbToolStatus::ConnectKVM {
                    // 连接到NanoKVM，开始系统操作线程
                    log("连接到NanoKVM，开始系统操作线程");
                    spawn(serial_system_operation_task());
                }
            } else {
                // 删除定时器
                if let Some(handle) = none_resv_timer_handle {
                    if delete_timer(handle).await {
                        none_resv_timer_handle = None;
                    }
                }
            }
        }

        match last_connection_status {
            UsbToolStatus::Unknown => {
                // log("USB工具初始化中...");
            }
            UsbToolStatus::Disconnected => {
                // 处理断开状态
            }
            UsbToolStatus::Connected => {
                // 检测是否连接KVM
                let received = serial_receive_clean().await;
                if !received.is_empty() {
                    // 清零计时器
                    if let Some(handle) = none_resv_timer_handle {
                        reset_timer(handle).await;
                    }
                    // 更新全局状态为连接到NanoKVM
                    if status == UsbToolStatus::Connected {
                        let mut status_guard = USB_TOOL_STATUS.lock().await;
                        *status_guard = UsbToolStatus::ConnectKVM;
                    }
                    // let received = serial_receive_clean().await;
                    log(&format!("接收到数据: {}", received));
                }

                // 若计时器超过10s，且状态为连接到NanoKVM，则更新状态为已连接
                if let Some(handle) = none_resv_timer_handle {
                    if let Some(timer_value) = check_timer(handle).await {
                        // log(&format!("定时器值: {}", timer_value));
                        if timer_value == OVERTIME_ENTER {
                            serial_send("\r\n").await;
                            // 等待1s，确保NanoKVM响应
                            sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                        if timer_value >= OVERTIME_LOST_KVM {
                            let mut status_guard = USB_TOOL_STATUS.lock().await;
                            *status_guard = UsbToolStatus::Connected;
                            // log("NanoKVM已断开");
                        }
                    }
                }
            }
            _ => {
                // 连接到KVM，不需要在这里操作
            }
        }
        
        // 短暂休眠，避免CPU占用过高
        sleep(Duration::from_millis(100)).await;
    }
}

// 检测USB工具是否已经连接
pub async fn is_usb_tool_connected() -> bool {
    let status = USB_TOOL_STATUS.lock().await;
    *status == UsbToolStatus::Connected || *status == UsbToolStatus::ConnectKVM
}

// 串口任务线程函数
pub fn spawn_serial_task() {
    spawn(async move {
        // 启动串口管理线程
        spawn(async move {
            serial_management_task().await;
        });
        // 启动串口数据管理线程
        spawn(async move {
            serial_data_management_task().await;
        });
        
        // 循环执行以下任务：发送"test"、接收数据、sleep 1秒
        loop {
            // serial_send("test").await;
            // let received = serial_receive().await; 
            sleep(Duration::from_secs(1)).await;
        }
    });
}