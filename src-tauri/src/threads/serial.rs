use serialport::{SerialPortType};
use tokio_serial::{SerialStream, new, SerialPortBuilderExt};
use std::time::Duration;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ErrorKind};
use tokio::time::{sleep, timeout};
use tauri::async_runtime::spawn;
use lazy_static::lazy_static;

// 定义PID/VID和波特率
const TARGET_VID: u16 = 0x1a86;
const TARGET_PID: u16 = 0x55d3;
const SERIAL_BAUD_RATE: u32 = 115200;

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = false;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("{}", msg);
    }
}

// 定义全局收发队列
lazy_static! {
    pub static ref SEND_QUEUE: Mutex<Option<mpsc::Sender<Vec<u8>>>> = Mutex::new(None);
    pub static ref RECEIVE_QUEUE: Mutex<Option<mpsc::Receiver<Vec<u8>>>> = Mutex::new(None);
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
        match receive_queue.recv().await {
            Some(data) => String::from_utf8_lossy(&data).to_string(),
            None => String::from("接收队列已关闭"),
        }
    } else {
        String::from("接收队列未初始化")
    }
}

// 串口管理线程函数
async fn serial_management_task() {
    let mut serial_port: Option<Arc<Mutex<SerialStream>>> = None;
    
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
            log("检查串口连接状态...");
            if let Some(port_name) = scan_serial_port(TARGET_PID, TARGET_VID) {
                log(&format!("找到串口: {}", port_name));
                if let Some(port) = connect_serial_port(&port_name).await {
                    log("成功连接串口");
                    serial_port = Some(port);
                } else {
                    log("连接串口失败");
                }
            } else {
                log("未找到目标串口");
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

// 串口任务线程函数
pub fn spawn_serial_task() {
    spawn(async move {
        // 启动串口管理线程
        spawn(async move {
            serial_management_task().await;
        });
        
        // 循环执行以下任务：发送"test"、接收数据、sleep 1秒
        loop {
            serial_send("test").await;
            let received = serial_receive().await;
            log(&format!("接收到数据: {}", received));
            sleep(Duration::from_secs(1)).await;
        }
    });
}