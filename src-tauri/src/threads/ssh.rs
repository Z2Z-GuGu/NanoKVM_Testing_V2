use ssh2::Session;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;
use tokio::task;

const HOST: &str = "192.168.1.109";
// const HOST: &str = "192.168.1.15";
const USER: &str = "root";
const PASSWORD: &str = "sipeed"; // 密码认证

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = true;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[ssh]{}", msg);
    }
}

pub async fn ssh_execute_command(command: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let command = command.to_string();
    
    task::spawn_blocking(move || {
        // 建立TCP连接
        let tcp = TcpStream::connect(format!("{}:22", HOST))?;
        
        // 创建SSH会话
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;
        
        // 使用密码认证
        session.userauth_password(USER, PASSWORD)?;
        
        if !session.authenticated() {
            return Err("SSH 认证失败".to_string().into());
        }
        
        log(&format!("✅ SSH 连接成功！执行命令: {}", command));
        
        // 执行命令
        let mut channel = session.channel_session()?;
        channel.exec(&command)?;
        
        // 读取命令输出
        let mut output = Vec::new();
        channel.read_to_end(&mut output)?;
        
        // 等待命令执行完成并获取退出状态
        let exit_status = channel.exit_status()?;
        
        // 关闭通道
        channel.send_eof()?;
        channel.wait_eof()?;
        channel.wait_close()?;
        
        // 转换输出为字符串
        let output_str = String::from_utf8(output)?;
        
        // 返回结果
        if exit_status == 0 {
            Ok(output_str)
        } else {
            Err(format!("命令执行失败，退出状态: {}\n输出: {}", exit_status, output_str).into())
        }
    }).await?
}

// 执行命令判断是否成功，返回结果包含是否成功和命令输出
pub async fn ssh_execute_command_check_success(command: &str, success_keyword: &str) -> Result<(bool, String), Box<dyn std::error::Error + Send + Sync>> {
    match ssh_execute_command(command).await {
        Ok(output) => {
            let success = output.contains(success_keyword);
            if success {
                // log(&format!("命令执行成功，包含关键词: {}", success_keyword));
            } else {
                // log(&format!("命令执行失败，不包含关键词: {}", success_keyword));
            }
            Ok((success, output))
        }
        Err(e) => {
            log(&format!("SSH命令执行失败: {}", e));
            Err(e)
        }
    }
}

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
//     println!("🔗 尝试连接到 {}@{}...", user, host);
    
//     // 使用 tokio::spawn_blocking 在异步上下文中运行所有同步的 SSH 操作
//     let _result: Result<(), Box<dyn std::error::Error + Send + Sync>> = tokio::task::spawn_blocking(move || {
//         // 建立TCP连接
//         let tcp = TcpStream::connect(format!("{}:22", host))?;
        
//         // 创建SSH会话
//         let mut session = Session::new()?;
//         session.set_tcp_stream(tcp);
//         session.handshake()?;
        
//         // 使用密码认证
//         session.userauth_password(user, password)?;
        
//         if !session.authenticated() {
//             return Err("SSH 认证失败".to_string().into());
//         }
        
//         println!("✅ SSH 连接成功！");
        
//         // 执行命令 - 就像在终端里一样
//         println!("\n📝 执行命令: ls -la /root");
//         {
//             let mut channel = session.channel_session()?;
//             channel.exec("ls -la /root")?;
            
//             let mut output = Vec::new();
//             channel.read_to_end(&mut output)?;
//             let exit_status = channel.exit_status()?;
            
//             channel.send_eof()?;
//             channel.wait_eof()?;
//             channel.wait_close()?;
            
//             let output_str = String::from_utf8(output)?;
//             println!("输出:\n{}", output_str);
//             if exit_status != 0 {
//                 println!("命令执行状态: {}", exit_status);
//             }
//         }
        
//         // 执行更多命令...
//         println!("\n📝 执行命令: pwd");
//         {
//             let mut channel = session.channel_session()?;
//             channel.exec("pwd")?;
            
//             let mut current_dir = Vec::new();
//             channel.read_to_end(&mut current_dir)?;
//             let exit_status = channel.exit_status()?;
            
//             channel.send_eof()?;
//             channel.wait_eof()?;
//             channel.wait_close()?;
            
//             let current_dir_str = String::from_utf8(current_dir)?;
//             println!("当前目录: {}", current_dir_str);
//             if exit_status != 0 {
//                 println!("命令执行状态: {}", exit_status);
//             }
//         }
        
//         // 创建测试目录
//         println!("\n📁 创建测试目录...");
//         {
//             let mut channel = session.channel_session()?;
//             channel.exec("mkdir -p /root/ssh-test")?;
            
//             let mut mkdir_output = Vec::new();
//             channel.read_to_end(&mut mkdir_output)?;
//             let exit_status = channel.exit_status()?;
            
//             channel.send_eof()?;
//             channel.wait_eof()?;
//             channel.wait_close()?;
            
//             let mkdir_output_str = String::from_utf8(mkdir_output)?;
//             if exit_status == 0 {
//                 println!("✅ 测试目录创建成功");
//             } else {
//                 println!("❌ 目录创建失败: {}", mkdir_output_str);
//             }
//         }
        
//         // 使用 SFTP 上传文件
//         println!("\n📤 使用 SFTP 上传文件...");
//         {
//             let sftp = session.sftp()?;
            
//             // 创建本地测试文件
//             std::fs::create_dir_all("../test")?;
//             std::fs::write("../test/hello.txt", "Hello from Rust SSH!")?;
            
//             // 通过SFTP上传文件
//             let local_file = "../test/hello.txt";
//             let remote_file = "/root/ssh-test/hello.txt";
            
//             match std::fs::File::open(local_file) {
//                 Ok(mut file) => {
//                     let remote_file_path = Path::new(remote_file);
//                     let mut remote_file = sftp.create(remote_file_path)?;
                    
//                     // 复制文件内容
//                     let mut buffer = Vec::new();
//                     file.read_to_end(&mut buffer)?;
//                     remote_file.write_all(&buffer)?;
                    
//                     println!("文件上传成功: {}", remote_file_path.display());
//                 }
//                 Err(e) => {
//                     println!("文件上传失败: {}", e);
//                 }
//             }
//         }
        
//         // 通过 SSH 重命名
//         println!("🔄 重命名文件夹...");
//         {
//             let mut channel = session.channel_session()?;
//             channel.exec("mv /root/test /root/ssh-test")?;
            
//             let mut rename_output = Vec::new();
//             channel.read_to_end(&mut rename_output)?;
//             let exit_status = channel.exit_status()?;
            
//             channel.send_eof()?;
//             channel.wait_eof()?;
//             channel.wait_close()?;
            
//             let rename_output_str = String::from_utf8(rename_output)?;
//             if exit_status == 0 {
//                 println!("✅ 重命名为 ssh-test 成功！");
                
//                 // 验证
//                 println!("📋 验证结果:");
//                 {
//                     let mut channel = session.channel_session()?;
//                     channel.exec("ls -la /root/ssh-test")?;
                    
//                     let mut verify_output = Vec::new();
//                     channel.read_to_end(&mut verify_output)?;
//                     let exit_status = channel.exit_status()?;
                    
//                     channel.send_eof()?;
//                     channel.wait_eof()?;
//                     channel.wait_close()?;
                    
//                     let verify_output_str = String::from_utf8(verify_output)?;
//                     println!("{}", verify_output_str);
//                     if exit_status != 0 {
//                         println!("验证命令执行状态: {}", exit_status);
//                     }
//                 }
//             } else {
//                 println!("❌ 重命名失败: {}", rename_output_str);
//             }
//         }
        
//         println!("\n🔌 SSH 操作完成");
        
//         Ok(())
//     }).await?;
    
//     Ok(())
// }