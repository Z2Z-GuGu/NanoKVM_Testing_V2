use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use tauri::async_runtime::spawn;
use tauri::AppHandle;
use tokio::time::{sleep, Duration};

// 日志控制：0=开启日志，1=关闭日志
const LOG_DISABLE: u8 = 1;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_DISABLE == 0 {
        println!("[camera.rs] {}", msg);
    }
}

// USB摄像头筛选条件（可根据实际情况修改）
const TARGET_VID: Option<u16> = None; // 设置为None表示匹配所有VID
const TARGET_PID: Option<u16> = None; // 设置为None表示匹配所有PID

// 摄像头状态枚举
#[derive(Debug, Clone, Copy)]
pub enum CameraStatus {
    Disconnected, // 未连接
    NoImage,      // 无画面（全黑）
    HasImage,     // 有画面
}

// 全局状态
static CAMERA_RUNNING: AtomicBool = AtomicBool::new(false);
// 使用AtomicI32存储状态，0: Disconnected, 1: NoImage, 2: HasImage
static CAMERA_STATUS: AtomicI32 = AtomicI32::new(0);

// 设置摄像头状态
fn set_camera_status(status: CameraStatus) {
    CAMERA_STATUS.store(status as i32, Ordering::Relaxed);
}

// 摄像头检测任务 - 模拟实现
// 注意：这是一个模拟实现，实际项目中需要替换为真正的摄像头检测代码
async fn camera_detection_task(_app_handle: AppHandle) {
    log("启动摄像头检测任务");
    loop {
        // 模拟摄像头检测逻辑
        // 这里可以根据实际需求替换为真正的摄像头检测代码
        
        // 模拟摄像头未连接状态
        // 在实际应用中，这里应该实现：
        // 1. 枚举可用摄像头设备
        // 2. 根据TARGET_VID和TARGET_PID筛选目标摄像头
        // 3. 尝试打开摄像头并检测画面
        log("检测到摄像头状态：Disconnected");
        set_camera_status(CameraStatus::Disconnected);
        
        // 每秒钟检查一次摄像头状态
        sleep(Duration::from_secs(1)).await;
    }
}

// 启动摄像头功能线程
pub fn spawn_camera_task(app_handle: AppHandle) {
    if CAMERA_RUNNING.swap(true, Ordering::Relaxed) {
        // 线程已经在运行
        log("摄像头线程已经在运行，忽略启动请求");
        return;
    }

    log("启动摄像头功能线程");
    spawn(async move {
        camera_detection_task(app_handle).await;
        // 线程结束时重置状态
        log("摄像头检测任务结束，重置状态");
        CAMERA_RUNNING.store(false, Ordering::Relaxed);
        set_camera_status(CameraStatus::Disconnected);
    });
}

// 查询摄像头状态的公共函数
pub fn get_camera_status() -> CameraStatus {
    let status_code = CAMERA_STATUS.load(Ordering::Relaxed);
    log(&format!("查询摄像头状态：{}", status_code));
    match status_code {
        0 => CameraStatus::Disconnected,
        1 => CameraStatus::NoImage,
        2 => CameraStatus::HasImage,
        _ => CameraStatus::Disconnected, // 默认状态
    }
}
