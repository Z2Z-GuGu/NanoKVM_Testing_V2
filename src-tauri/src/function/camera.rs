use nokhwa::{
    Camera, 
    utils::{
        CameraIndex, 
        RequestedFormat,
        RequestedFormatType
    }
};
use nokhwa::pixel_format::RgbFormat;
use std::thread;
use std::time::Duration;
use lazy_static::lazy_static;
use tokio::sync::Mutex;
use tokio;

// 摄像头状态枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraStatus {
    Disconnected,    // 未连接
    Connected,       // 已连接
    HasImage        // 有画面
}

// 全局摄像头状态变量
lazy_static! {
    static ref CAMERA_STATUS: Mutex<CameraStatus> = Mutex::new(CameraStatus::Disconnected);
}

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = false;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[camera]{}", msg);
    }
}

// 检查图像是否为全黑
fn is_image_black(image_data: &[u8]) -> bool {
    // RGB图像每个像素有3个字节(R, G, B)
    // 如果所有像素的RGB值都接近0，则认为图像是全黑的
    
    // 设置一个阈值，如果RGB值都小于这个阈值，认为是黑色
    let black_threshold = 10; // 允许一些噪声
    
    // 检查每个像素
    for chunk in image_data.chunks(3) {
        if chunk.len() == 3 {
            let r = chunk[0];
            let g = chunk[1];
            let b = chunk[2];
            
            // 如果任何一个颜色分量超过阈值，则不是全黑
            if r > black_threshold || g > black_threshold || b > black_threshold {
                return false;
            }
        }
    }
    
    // 所有像素都是黑色的
    true
}

pub fn spawn_camera_task() {
    thread::spawn(move || {
        log("开始摄像头监控程序...");
        
        let mut camera: Option<Camera> = None;
        let camera_index = CameraIndex::Index(0);
        let runtime = tokio::runtime::Runtime::new().unwrap();
        
        // 主循环
        loop {
            // 检查摄像头是否连接
            if camera.is_none() {
                log("等待摄像头连接...");
                runtime.block_on(async {
                    let mut status = CAMERA_STATUS.lock().await;
                    *status = CameraStatus::Disconnected;
                });
                
                // 尝试连接摄像头
                match Camera::new(camera_index.clone(), RequestedFormat::new::<RgbFormat>(RequestedFormatType::HighestFrameRate(30))) {
                    Ok(cam) => {
                        log("摄像头已连接！");
                        camera = Some(cam);
                        runtime.block_on(async {
                            let mut status = CAMERA_STATUS.lock().await;
                            *status = CameraStatus::Connected;
                        });
                    },
                    Err(_) => {
                        log("摄像头未连接，等待1秒后重试...");
                        thread::sleep(Duration::from_secs(1));
                        continue;
                    }
                }
            }
            
            // 摄像头已连接，定期检查图像
            if let Some(ref mut cam) = camera {
                match cam.frame() {
                    Ok(frame) => {
                        // 将帧转换为RGB图像数据
                        match frame.decode_image::<RgbFormat>() {
                            Ok(image_data) => {
                                // 检查图像是否为全黑
                                let is_black = is_image_black(&image_data);
                                if is_black {
                                    log("警告: 捕捉到的图像是全黑的！");
                                    runtime.block_on(async {
                                        let mut status = CAMERA_STATUS.lock().await;
                                        *status = CameraStatus::Connected;
                                    });
                                } else {
                                    log("图像正常，不是全黑的");
                                    runtime.block_on(async {
                                        let mut status = CAMERA_STATUS.lock().await;
                                        *status = CameraStatus::HasImage;
                                    });
                                }
                            },
                            Err(_) => {
                                log("图像解码失败！");
                                runtime.block_on(async {
                                    let mut status = CAMERA_STATUS.lock().await;
                                    *status = CameraStatus::Connected;
                                });
                            }
                        }
                        
                        // 等待1秒
                        thread::sleep(Duration::from_secs(1));
                    },
                    Err(_) => {
                        log("摄像头连接断开！");
                        camera = None;
                        runtime.block_on(async {
                            let mut status = CAMERA_STATUS.lock().await;
                            *status = CameraStatus::Disconnected;
                        });
                        // 等待1秒后尝试重新连接
                        thread::sleep(Duration::from_secs(1));
                    }
                }
            }
        }
    });
}

// 获取当前摄像头状态（供其他线程调用）
pub async fn get_camera_status() -> CameraStatus {
    let status = CAMERA_STATUS.lock().await;
    *status
}