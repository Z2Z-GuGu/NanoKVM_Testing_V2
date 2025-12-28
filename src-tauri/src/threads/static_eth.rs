use ipconfig;
use std::process::Command;

use std::thread;
use std::time::Duration;

// 静态IP使能
pub const STATIC_IP_ENABLE: bool = true;

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = false;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[static_eth]{}", msg);
    }
}

// 必须使用管理员权限运行

fn set_static_ip(eth_name: &str, ip: &str, mask: &str) -> std::io::Result<()> {
    // netsh interface ip set address name="Ethernet 2" static 172.168.100.1 255.255.255.0

    let mut command = Command::new("netsh");
    
    // 构建命令参数
    command.args(&[
        "interface", "ip", "set", "address",
        // &format!("name=\"{}\"", eth_name),  // 注意这里需要用双引号包裹
        &format!("name={}", eth_name),
        "static",
        ip,
        mask
    ]);

    // 打印命令（调试用）
    // println!("执行命令: {:?}", command);
    
    // 执行命令
    let output = command.output()?;
    
    if output.status.success() {
        log("IP配置成功");
    } else {
        log(&format!("IP配置失败: {}", String::from_utf8_lossy(&output.stderr)));
    }
    
    Ok(())
}

pub fn set_static_ip_for_testing(static_ip: &str) -> Result<bool, Box<dyn std::error::Error>>{
    let mut eth_name = String::new();
    let ip = static_ip;
    let mask = "255.255.255.0";

    for adapter in ipconfig::get_adapters()? {
        // 如果适配器friendly_name有“以太网”或“Ethernet”字眼，硬件描述中包含“Realtek”字眼，并且IP地址中不包含“192”字眼，则输出“++++{}+++++”friendly_name
        if adapter.friendly_name().contains("以太网") || adapter.friendly_name().contains("Ethernet") {
            if adapter.description().contains("Realtek") && !adapter.ip_addresses().iter().any(|ip| ip.to_string().contains("192")) {
                // 仅保存名字字符串，直接跳出for循环
                eth_name = adapter.friendly_name().to_string();
                break;
            }
        }
    }
    // 如果eth_name为空字符串，则输出“未找到以太网适配器”
    if eth_name.is_empty() {
        log("未找到以太网适配器");
    } else {
        log(&format!("以太网适配器名称: {}", eth_name));
        set_static_ip(&eth_name, ip, mask)?;
    }
    Ok(true)
}