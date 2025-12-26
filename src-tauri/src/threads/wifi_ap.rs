use std::sync::mpsc;
use std::thread;
use std::time::Duration;
// use anyhow::Result;
use wifidirect_legacy_ap::WlanHostedNetworkHelper;
use tauri::async_runtime::{spawn, JoinHandle};
use tokio::time::sleep;


const AP_ENABLE: bool = true;          // 是否真的开启wifi

// const SSID: &str = "RustHotspot";      // 热点名称
// const PASSWORD: &str = "Rust123456";   // 密码（至少8个字符）

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = true;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[wifi-ap]{}", msg);
    }
}

pub fn spawn_wifi_ap(ssid: &str, password: &str) -> JoinHandle<()> {
    let ssid = ssid.to_string();
    let password = password.to_string();
    spawn(async move {
        log("正在启动WiFi热点...");
        if AP_ENABLE {
            let (tx, rx) = mpsc::channel::<String>();
    
            // 尝试创建并启动热点
            let helper_result = WlanHostedNetworkHelper::new(&ssid, &password, tx);
            
            match helper_result {
                Ok(wlan_hosted_network_helper) => {
                    println!("✓ 热点创建成功！");
                    println!("热点正在运行，按 Ctrl+C 可提前停止\n");
                    
                    // 启动状态监控线程
                    let status_thread = thread::spawn(move || {
                        while let Ok(message) = rx.recv() {
                            println!("[状态] {}", message);
                            
                            // 检查是否收到停止消息
                            if message.contains("停止") || message.contains("stop") {
                                break;
                            }
                        }
                        println!("状态监听线程结束");
                    });

                    loop {
                        sleep(Duration::from_secs(1)).await;
                    }
                
                    
                    // // 停止热点
                    // match wlan_hosted_network_helper.stop() {
                    //     Ok(_) => println!("✓ 热点已停止"),
                    //     Err(e) => println!("✗ 停止热点时出错: {:?}", e),
                    // }
                    
                    // // 等待状态线程结束
                    // let _ = status_thread.join();
                    
                    // println!("程序执行完成");
                }
                Err(e) => {
                    log(&format!("创建热点失败: {:?}", e));
                }
            }
        }

        loop {
            sleep(Duration::from_secs(1)).await;
        }
    })
}
