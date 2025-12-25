use tauri::async_runtime::{spawn, JoinHandle};


use warp::Filter;
use bytes::Bytes;
use std::fs::File;
use std::io::Read;

use crate::threads::save::get_app_file_path;

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = true;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[server]{}", msg);
    }
}

pub fn spawn_file_server_task() -> JoinHandle<()> {
    spawn(async move {
        log("文件服务器任务开始");
        // 下载路由
        let download = warp::path("download")
            .and(warp::get())
            .and(warp::query::<DownloadParams>())
            .and_then(download_handler);

        // 上传路由
        let upload = warp::path("upload")
            .and(warp::post())
            .and(warp::body::bytes())
            .and_then(upload_handler);

        // 组合路由
        let routes = download
            .or(upload)
            .with(warp::cors().allow_any_origin());

        warp::serve(routes).run(([192, 168, 2, 201], 8080)).await;
    })
}

// 下载参数
#[derive(serde::Deserialize)]
struct DownloadParams {
    // size_mb: Option<usize>,
}

// 下载处理 - 读取固定文件
async fn download_handler(_params: DownloadParams) -> Result<impl warp::Reply, warp::Rejection> {
    // "C:\Users\BuGu\AppData\Local\NanoKVM-Testing\app\NanoKVM_Pro_Testing_V2_0.tar"
    // let file_path = "C:\\Users\\BuGu\\AppData\\Local\\NanoKVM-Testing\\app\\NanoKVM_Pro_Testing_V2_0.tar"
    let file_path = get_app_file_path();
    log(&format!("获取到的文件路径: {:?}", file_path));

    log("开始下载测试");
    
    // 读取文件内容
    let mut file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            log(&format!("❌ 无法打开文件: {}", e));
            return Err(warp::reject::not_found());
        }
    };
    
    let mut data = Vec::new();
    if let Err(e) = file.read_to_end(&mut data) {
        log(&format!("❌ 读取文件失败: {}", e));
        return Err(warp::reject::not_found());
    }
    
    log("✅ 下载完成");
    
    Ok(data)
}

// 上传处理 - 虚拟内存，不会真的存到文件系统
async fn upload_handler(
    body: Bytes
) -> Result<impl warp::Reply, warp::Rejection> {
    log("📤 开始接收上传数据...");
    
    let total_bytes = body.len();
    
    // 模拟网络传输延迟（基于数据大小）
    let simulated_delay_ms = (total_bytes as f64 / (1024.0 * 1024.0) * 10.0).max(10.0); // 每MB延迟10ms，最少10ms
    tokio::time::sleep(std::time::Duration::from_millis(simulated_delay_ms as u64)).await;
    
    log("✅ 上传完成");
    
    Ok(warp::reply::json(&serde_json::json!({
        "success": true,
        "message": "上传完成"
    })))
}