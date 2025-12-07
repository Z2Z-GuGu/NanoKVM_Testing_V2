use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use serialport::{SerialPortType, DataBits, Parity, StopBits, FlowControl, SerialPortBuilder};
use tauri::async_runtime::spawn;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_serial::SerialStream;

// USB设备筛选条件
const TARGET_VID: u16 = 0x1a86;
const TARGET_PID: u16 = 0x7523;

// 串口配置
const BAUD_RATE: u32 = 1500000;
const DATA_BITS: DataBits = DataBits::Eight;
const PARITY: Parity = Parity::None;
const STOP_BITS: StopBits = StopBits::One;

// 共享状态，用于在不同任务间传递串口连接
struct SerialState {
    port: Option<SerialStream>,
}

impl SerialState {
    fn new() -> Self {
        Self {
            port: None,
        }
    }
}

// 发送消息到前端
fn send_to_frontend(app_handle: &AppHandle, message: &str) {
    app_handle.emit("terminal-output", message).unwrap();
}

// 扫描并连接串口设备
async fn scan_and_connect_serial(app_handle: &AppHandle, state: &Arc<Mutex<SerialState>>) -> bool {
    // 扫描所有可用的串口设备
    let ports = match serialport::available_ports() {
        Ok(ports) => ports,
        Err(e) => {
            eprintln!("Failed to scan serial ports: {}", e);
            return false;
        }
    };

    // 筛选目标设备
    for port_info in ports {
        // 检查USB VID和PID
        match port_info.port_type {
            SerialPortType::UsbPort(usb_info) => {
                if usb_info.vid == TARGET_VID && usb_info.pid == TARGET_PID {
                // 尝试连接串口
                match connect_serial(app_handle, state, &port_info.port_name).await {
                    Ok(()) => return true,
                    Err(e) => {
                        eprintln!("Failed to connect to serial port {}: {}", port_info.port_name, e);
                        continue;
                    }
                }
                }
            },
            _ => continue,
        }
    }

    false
}

// 连接到指定的串口设备
async fn connect_serial(
    app_handle: &AppHandle,
    state: &Arc<Mutex<SerialState>>,
    port_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 配置串口
    let builder = serialport::new(port_name, BAUD_RATE)
        .data_bits(DATA_BITS)
        .parity(PARITY)
        .stop_bits(STOP_BITS)
        .flow_control(FlowControl::None);

    // 打开串口
    let port = SerialStream::open(&builder)?;
    
    // 更新共享状态
    let mut state_guard = state.lock().await;
    state_guard.port = Some(port);
    
    // 发送连接成功消息到前端
    send_to_frontend(app_handle, "[connected]\n");
    
    Ok(())
}

// 接收串口数据并转发到前端
async fn receive_serial_data(app_handle: &AppHandle, state: &Arc<Mutex<SerialState>>) {
    loop {
        let mut buffer = vec![0; 1024];
        
        {
            let mut state_guard = state.lock().await;
            
            if let Some(port) = &mut state_guard.port {
                match port.read(&mut buffer).await {
                    Ok(n) if n > 0 => {
                        // 将接收到的数据转换为字符串并发送到前端
                        if let Ok(data) = String::from_utf8(buffer[..n].to_vec()) {
                            send_to_frontend(app_handle, &data);
                        }
                    }
                    Ok(_) => {
                        // 读取零字节，可能是设备断开
                        state_guard.port = None;
                        send_to_frontend(app_handle, "[disconnected]\n");
                    }
                    Err(e) => {
                        // 读取错误，可能是设备断开
                        eprintln!("Serial read error: {}", e);
                        state_guard.port = None;
                        send_to_frontend(app_handle, "[disconnected]\n");
                    }
                }
            }
        }
        
        // 短暂休眠以避免CPU占用过高
        sleep(Duration::from_millis(10)).await;
    }
}

// 发送数据到串口设备
pub async fn send_serial_data(state: &Arc<Mutex<SerialState>>, data: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut state_guard = state.lock().await;
    
    if let Some(port) = &mut state_guard.port {
        port.write_all(data.as_bytes()).await?;
    }
    
    Ok(())
}

// 启动串口功能线程
pub fn spawn_serial_task(app_handle: AppHandle) {
    // 创建共享状态
    let state = Arc::new(Mutex::new(SerialState::new()));
    
    // 克隆状态用于接收任务
    let state_clone = state.clone();
    let app_handle_clone = app_handle.clone();
    
    // 启动接收数据任务
    spawn(async move {
        receive_serial_data(&app_handle_clone, &state_clone).await;
    });
    
    // 启动扫描和连接任务
    spawn(async move {
        loop {
            // 尝试扫描并连接串口设备
            let connected = scan_and_connect_serial(&app_handle, &state).await;
            
            // 如果连接成功，等待一段时间后再次检查
            if connected {
                sleep(Duration::from_secs(5)).await;
            } else {
                // 如果连接失败，短暂休眠后再次尝试
                sleep(Duration::from_millis(500)).await;
            }
        }
    });
}
