use anyhow::{Result, Context};
use reqwest::Client;
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use tauri::async_runtime::{spawn};
use tauri::{AppHandle};
use std::time::Duration;
use std::thread;
use crate::threads::update_state::{set_server_state, set_upload_count};
use crate::threads::save::{get_unuploaded_num, get_one_unuploaded_file_path, rm_from_unuploaded};

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = false;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[upload]{}", msg);
    }
}

// 从JSON文件读取并上传数据
pub async fn upload_from_json_file(file_path: &str) -> Result<bool> {
    // 1. 读取JSON文件
    let json_content = fs::read_to_string(file_path)
        .context(format!("Failed to read file: {}", file_path))?;
    
    // 2. 解析JSON
    let json_data: Value = serde_json::from_str(&json_content)
        .context("Failed to parse JSON")?;
    
    // 3. 提取必填字段
    let device_info = &json_data["device_info"];
    let uid = device_info["soc_uid"]
        .as_str()
        .context("Missing soc_uid in device_info")?
        .to_string();
    
    let serial = device_info["serial"]
        .as_str()
        .context("Missing serial in device_info")?
        .to_string();
    
    let hardware = device_info["hardware"]
        .as_str()
        .context("Missing hardware in device_info")?
        .to_string();

    let test_pass = device_info["test_pass"]
        .as_bool()
        .context("Missing test_pass in device_info")?;
    
    // 4. 提取测试结果
    let test_content = &json_data["test_content"];
    
    // 5. 使用 serde_json::json! 宏构建JSON，这是关键修复
    let mut request_body = json!({
        "uid": uid,
        "serial": serial,
        "hardware": hardware,
    });
    
    // 解析非必要项目并添加到json
    add_test_fields(&mut request_body, test_content);
    
    // 打印查看（可选）
    // log(&format!("JSON请求体: {}", serde_json::to_string_pretty(&request_body)?));
    
    // 7. 发送请求
    let mut final_result = true;
    let data_result = upload_json_data(&request_body).await;
    if test_pass {
        final_result = upload_test_results(&serial).await;
    }
    
    Ok(final_result && data_result)
}

fn add_test_fields(request_body: &mut Value, test_content: &Value) {
    if let Value::Object(map) = request_body {
        let test_fields = [
            "app", "atx", "emmc", "eth", "lt6911", "lt86102", "rotary",
            "screen", "sdcard", "touch", "uart", "usb", "wifi", "ws2812",
        ];
        
        for field in test_fields.iter() {
            if let Some(value) = test_content.get(*field) {
                if let Some(str_value) = value.as_str() {
                    map.insert(
                        (*field).to_string(),
                        Value::String(str_value.to_string()),
                    );
                }
            }
        }
    }
}

// 上传JSON数据到服务器
async fn upload_json_data(request_body: &Value) -> bool {
    let client = Client::new();
    let url = "https://maixvision.sipeed.com/api/v1/nanokvm/test-items";
    
    let response = match client
        .post(url)
        .header("token", "MaixVision2024")
        .json(request_body)  // 直接传递 Value 类型，reqwest 会正确处理
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            log(&format!("Failed to send request: {}", e));
            return false;
        }
    };
    
    // 解析响应
    let json_response = match response.json::<Value>().await {
        Ok(json) => json,
        Err(e) => {
            log(&format!("Failed to parse response: {}", e));
            return false;
        }
    };
    
    let code = match json_response["code"].as_i64() {
        Some(c) => c as i32,
        None => {
            log(&format!("Missing code in response"));
            return false;
        }
    };
    
    let msg = match json_response["msg"].as_str() {
        Some(m) => m.to_string(),
        None => {
            log(&format!("Missing msg in response"));
            return false;
        }
    };
    
    if code == 0 {
        log(&format!("上传成功: code={}, msg={}", code, msg));
        true
    } else {
        log(&format!("上传失败: code={}, msg={}", code, msg));
        false
    }
}

// 上传测试结果到服务器
async fn upload_test_results(serial: &str) -> bool {
    let client = Client::new();
    let url = "https://maixvision.sipeed.com/api/v1/nanokvm/test-result";

    let test_status = "pass";
    let request_body = json!({
        "serial": serial,
        "status": test_status,
    });
    
    let response = match client
        .post(url)
        .header("token", "MaixVision2024")
        .header("passwd", "Sipeed.NanoKVM@25")
        .json(&request_body)  // 直接传递 Value 类型，reqwest 会正确处理
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(e) => {
            log(&format!("Failed to send request: {}", e));
            return false;
        }
    };
    
    // 解析响应
    let json_response = match response.json::<Value>().await {
        Ok(json) => json,
        Err(e) => {
            log(&format!("Failed to parse response: {}", e));
            return false;
        }
    };
    
    let code = match json_response["code"].as_i64() {
        Some(c) => c as i32,
        None => {
            log(&format!("Missing code in response"));
            return false;
        }
    };
    
    let msg = match json_response["msg"].as_str() {
        Some(m) => m.to_string(),
        None => {
            log(&format!("Missing msg in response"));
            return false;
        }
    };
    
    if code == 0 || code == -4 {
        log(&format!("上传成功: code={}, msg={}", code, msg));
        true
    } else {
        log(&format!("上传失败: code={}, msg={}", code, msg));
        false
    }
}

// #[tokio::main]
// async fn main() -> Result<()> {
//     if upload_from_json_file("NeaZ10003.json").await? {
//         log(&format!("所有上传操作成功完成"));
//     } else {
//         log(&format!("部分或全部上传操作失败"));
//     }
    
//     Ok(())
// }
pub fn spawn_upload_task(app_handle: AppHandle) {
    log(&format!("开始上传线程"));
    spawn(async move {
        loop {
            log(&format!("开始检查未上传文件"));
            let mut unuploaded_num = get_unuploaded_num() as u64;
            set_upload_count(app_handle.clone(), unuploaded_num);
            while unuploaded_num > 0 {
                unuploaded_num = get_unuploaded_num() as u64;
                set_upload_count(app_handle.clone(), unuploaded_num);
                if unuploaded_num > 0 {
                    let file_path = get_one_unuploaded_file_path().unwrap();
                    match upload_from_json_file(&file_path).await {
                        Ok(true) => {
                            log(&format!("上传文件 {} 成功", file_path));
                            let file_name = PathBuf::from(&file_path).file_name().unwrap().to_str().unwrap().to_string();
                            let _ = rm_from_unuploaded(&file_name);

                            set_server_state(app_handle.clone(), true);
                        },
                        Ok(false) => {
                            log(&format!("上传文件 {} 失败", file_path));
                            set_server_state(app_handle.clone(), false);
                        },
                        Err(e) => {
                            log(&format!("上传文件 {} 出错: {:?}", file_path, e));
                            set_server_state(app_handle.clone(), false);
                        }
                    }
                }
                thread::sleep(Duration::from_secs(1));
            }


            thread::sleep(Duration::from_secs(10));
        }
    });
}