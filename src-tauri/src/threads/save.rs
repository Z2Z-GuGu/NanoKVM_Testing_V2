use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use toml;
use lazy_static::lazy_static;
use std::sync::Mutex;
use chrono::Local;
use chrono::Datelike;

lazy_static! {
    /// 全局存储应用程序根路径
    static ref APP_ROOT: Mutex<Option<PathBuf>> = Mutex::new(None);
}

/// 应用程序配置（[application] 部分）
#[derive(Deserialize, Debug)]
pub struct ApplicationConfig {
    pub machine_number: String,
}

/// 测试配置（[testing] 部分）
#[derive(Deserialize, Debug)]
pub struct TestingConfig {
    pub board_version: String,
    pub desktop_mode: String,
    pub eth_mod: String,
    pub eth_up_speed: u32,
    pub eth_down_speed: u32,
    pub wifi_up_speed: u32,
    pub wifi_down_speed: u32,
}

/// 完整配置结构
#[derive(Deserialize, Debug)]
pub struct AppConfig {
    pub application: ApplicationConfig,
    pub testing: TestingConfig,
}

/// 设备信息结构体
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DeviceInfo {
    pub serial: String,
    pub soc_uid: String,
    pub hardware: String,
    pub wifi_exist: bool,
    pub test_pass: bool,
    pub unuploaded: bool,
}

/// 测试内容结构体
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TestContent {
    pub app: String,
    pub atx: String,
    pub emmc: String,
    pub eth: String,
    pub lt6911: String,
    pub lt86102: String,
    pub rotary: String,
    pub screen: String,
    pub sdcard: String,
    pub touch: String,
    pub uart: String,
    pub usb: String,
    pub wifi: String,
    pub ws2812: String,
}

/// 测试日志条目结构体
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TestLogEntry {
    pub test_pass: bool,
    #[serde(flatten)]
    pub other_fields: std::collections::HashMap<String, String>,
}

/// 测试日志结构体
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TestLog {
    #[serde(flatten)]
    pub entries: std::collections::HashMap<String, TestLogEntry>,
}

/// 完整的JSON数据结构体
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TestData {
    pub device_info: DeviceInfo,
    pub test_content: TestContent,
    pub test_log: TestLog,
}

/// 在 AppData\Local 下初始化应用程序数据目录结构
/// 
/// # 参数
/// - `app_name`: 应用程序名称，如 "MyAPP"
/// 
/// # 返回
/// - `Ok(根路径)` 如果创建成功
/// - `Err(错误信息)` 如果创建失败
/// 
/// # 创建的目录结构
/// - `{app_name}/config/config.toml`      # 配置文件
/// - `{app_name}/data/unuploaded/`          # 上传文件目录
/// - `{app_name}/data/save/`              # 保存文件目录
pub fn init_appdata(app_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // 1. 获取 AppData\Local 基础路径
    let local_appdata = get_local_appdata()?;
    println!("AppData\\Local 路径: {}", local_appdata.display());
    
    // 2. 构建应用程序根目录
    let app_root = local_appdata.join(app_name);
    println!("应用程序根目录: {}", app_root.display());
    
    // 将应用程序根路径存储到全局变量
    *APP_ROOT.lock().unwrap() = Some(app_root.clone());
    
    // 3. 创建目录结构
    create_directory_structure(&app_root, app_name)?;
    
    // 4. 创建默认配置文件
    create_default_config()?;
    
    println!("\n✅ 目录结构初始化完成！");
    println!("📁 根目录: {}", app_root.display());
    
    Ok(app_root)
}

/// 获取 LOCALAPPDATA 环境变量路径
fn get_local_appdata() -> Result<PathBuf, Box<dyn std::error::Error>> {
    match std::env::var("LOCALAPPDATA") {
        Ok(path) => Ok(PathBuf::from(path)),
        Err(_) => Err("无法获取 LOCALAPPDATA 环境变量，请确保在Windows系统上运行".into()),
    }
}

/// 创建完整的目录结构
fn create_directory_structure(app_root: &Path, app_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 定义需要创建的目录
    let directories = vec![
        app_root.join("config"),
        app_root.join("data").join("unuploaded"),
        app_root.join("data").join("save"),
        app_root.join("app"),
    ];
    
    println!("\n📂 正在创建目录结构:");
    
    for dir in &directories {
        if !dir.exists() {
            match fs::create_dir_all(dir) {
                Ok(_) => println!("   ✓ 创建: {}", get_relative_path(dir, app_name)),
                Err(e) => {
                    eprintln!("   ✗ 创建失败 {}: {}", dir.display(), e);
                    return Err(format!("创建目录失败: {}", e).into());
                }
            }
        } else {
            println!("   • 已存在: {}", get_relative_path(dir, app_name));
        }
    }
    
    Ok(())
}

/// 将JSON文件从save目录复制到unuploaded目录，并设置uploaded为true
/// 
/// # 参数
/// - `serial`: 设备序列号，作为JSON文件名的索引
/// 
/// # 返回
/// - `Ok(())` 如果操作成功
/// - `Err(错误信息)` 如果操作失败
pub fn cp_to_unuploaded(serial: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root_path = get_app_root()?;
    
    // 构建源文件和目标文件路径
    let save_path = root_path.join("data").join("save").join(format!("{}.json", serial));
    let unuploaded_path = root_path.join("data").join("unuploaded").join(format!("{}.json", serial));
    
    // 检查源文件是否存在
    if !save_path.exists() {
        return Err(format!("源文件不存在: {}", save_path.display()).into());
    }
    
    // 读取源文件内容
    let content = fs::read_to_string(&save_path)?;
    let mut test_data: TestData = serde_json::from_str(&content)?;
    
    // 设置unuploaded为true（复制到unuploaded文件夹意味着没有上传）
    test_data.device_info.unuploaded = true;
    
    // 保存更新后的数据到源文件
    let updated_content = serde_json::to_string_pretty(&test_data)?;
    fs::write(&save_path, &updated_content)?;
    
    // 复制到unuploaded目录
    fs::write(&unuploaded_path, &updated_content)?;
    
    Ok(())
}

/// 将JSON文件从unuploaded目录删除，并设置uploaded为false
/// 
/// # 参数
/// - `serial`: 设备序列号，作为JSON文件名的索引
/// 
/// # 返回
/// - `Ok(())` 如果操作成功
/// - `Err(错误信息)` 如果操作失败
pub fn rm_from_unuploaded(serial: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root_path = get_app_root()?;
    
    // 构建源文件和目标文件路径
    let save_path = root_path.join("data").join("save").join(format!("{}.json", serial));
    let unuploaded_path = root_path.join("data").join("unuploaded").join(format!("{}.json", serial));
    
    // 检查save目录中的文件是否存在
    if !save_path.exists() {
        return Err(format!("save目录中的文件不存在: {}", save_path.display()).into());
    }
    
    // 读取源文件内容
    let content = fs::read_to_string(&save_path)?;
    let mut test_data: TestData = serde_json::from_str(&content)?;
    
    // 设置unuploaded为false（从unuploaded文件夹删除意味着已上传）
    test_data.device_info.unuploaded = false;
    
    // 保存更新后的数据到源文件
    let updated_content = serde_json::to_string_pretty(&test_data)?;
    fs::write(&save_path, &updated_content)?;
    
    // 删除unuploaded目录中的文件（如果存在）
    if unuploaded_path.exists() {
        fs::remove_file(&unuploaded_path)?;
    }
    
    Ok(())
}

/// 创建默认配置文件
fn create_default_config() -> Result<(), Box<dyn std::error::Error>> {
    let app_root = get_app_root()?;
    let config_file = app_root.join("config").join("config.toml");
    
    if config_file.exists() {
        println!("📄 配置文件已存在: {}", get_relative_path(&config_file, ""));
        return Ok(());
    }
    
    let config_content = generate_default_config();
    
    match fs::write(&config_file, &config_content) {
        Ok(_) => {
            println!("📄 创建配置文件: {}", get_relative_path(&config_file, ""));
            Ok(())
        }
        Err(e) => {
            eprintln!("✗ 创建配置文件失败: {}", e);
            Err(format!("创建配置文件失败: {}", e).into())
        }
    }
}

/// 生成默认配置文件内容
fn generate_default_config() -> String {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    
    format!(
        r#"# {app_name} 配置文件
# 自动生成于 {timestamp}

[application]
machine_number = "0"    # 用于设置测试主机的唯一标识符，填写"1"-"9"中的一个数字，"0"无效

[testing]
board_version = "F"     # 测试板版本，填写板子识别编号字母位，如板子编号为30126F，则填写"F"
desktop_mode = "dark"   # 桌面默认模式，可选 "light" 或 "dark"
eth_mod = "static"      # 以太网模式，可选 "static" 或 "router"，用以选择是否网线直连待测主机
eth_up_speed = 300      # 测试以太网上传速度，单位Mbps
eth_down_speed = 500    # 测试以太网下载速度，单位Mbps
wifi_up_speed = 10      # 测试WiFi上传速度，单位Mbps
wifi_down_speed = 10    # 测试WiFi下载速度，单位Mbps

# 注意：修改配置后需要重启应用程序生效
"#,
        app_name = "MyAPP",  // 这里使用硬编码，或者可以改为参数传递
        timestamp = timestamp
    )
}

/// 获取全局应用程序根路径
/// 
/// # 返回
/// - `Ok(PathBuf)` 如果应用程序已初始化
/// - `Err(错误信息)` 如果应用程序未初始化
fn get_app_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_root = APP_ROOT.lock().unwrap();
    match app_root.as_ref() {
        Some(path) => Ok(path.clone()),
        None => Err("应用程序根路径未初始化，请先调用 init_appdata".into()),
    }
}

/// 获取相对于应用程序根目录的相对路径（用于显示）
fn get_relative_path(path: &Path, app_name: &str) -> String {
    let full_path = path.to_string_lossy();
    if app_name.is_empty() {
        return full_path.to_string();
    }
    
    // 尝试提取相对路径部分
    let pattern = format!("\\{}\\", app_name);
    if let Some(pos) = full_path.find(&pattern) {
        let relative = &full_path[pos + app_name.len() + 1..];
        return format!("{}/{}", app_name, relative.replace("\\", "/"));
    }
    
    full_path.to_string()
}

/// 解析TOML配置文件
/// 
/// # 参数
/// - `config_path`: 配置文件路径
/// 
/// # 返回
/// - `Ok(AppConfig)` 如果解析成功
/// - `Err(错误信息)` 如果解析失败
pub fn parse_config(config_path: &Path) -> Result<AppConfig, Box<dyn std::error::Error>> {
    // 读取配置文件内容
    let config_content = fs::read_to_string(config_path)?;
    
    // 解析TOML内容到结构体
    let config: AppConfig = toml::from_str(&config_content)?;
    
    Ok(config)
}

/// 获取toml中指定参数的字符串（如果是u32整数则转换为字符串）
/// 
/// # 参数
/// - `section`: 配置节名称，如 "application" 或 "testing"
/// - `key`: 配置键名称
/// 
/// # 返回
/// - `Some(String)` 如果找到对应的配置值
/// - `None` 如果配置不存在或解析失败
pub fn get_config_str(section: &str, key: &str) -> Option<String> {
    // 获取应用程序根路径
    let app_root = get_app_root().ok()?;
    let config_path = app_root.join("config").join("config.toml");
    
    // 读取配置文件内容
    let config_content = fs::read_to_string(&config_path).ok()?;
    
    // 解析TOML内容到结构体
    let config: AppConfig = toml::from_str(&config_content).ok()?;

    // 获取指定section和key的字符串值，如果是u32整数则转换为字符串
    match section {
        "application" => match key {
            "machine_number" => Some(config.application.machine_number),
            _ => None,
        },
        "testing" => match key {
            "board_version" => Some(config.testing.board_version),
            "desktop_mode" => Some(config.testing.desktop_mode),
            "eth_mod" => Some(config.testing.eth_mod),
            "eth_up_speed" => Some(config.testing.eth_up_speed.to_string()),
            "eth_down_speed" => Some(config.testing.eth_down_speed.to_string()),
            "wifi_up_speed" => Some(config.testing.wifi_up_speed.to_string()),
            "wifi_down_speed" => Some(config.testing.wifi_down_speed.to_string()),
            _ => None,
        },
        _ => None,
    }
}

/// 设置测试状态
/// 
/// # 参数
/// - `serial`: 设备序列号，作为JSON文件名的索引
/// - `item`: 测试项目名称
/// - `status`: 测试状态
/// 
/// # 返回
/// - `Ok(())` 如果设置成功
/// - `Err(错误信息)` 如果设置失败
pub fn set_test_status(serial: &str, item: &str, status: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root_path = get_app_root()?;
    
    // 构建JSON文件路径
    let json_path = root_path.join("data").join("save").join(format!("{}.json", serial));
    
    // 读取现有数据或创建新数据
    let mut test_data = if json_path.exists() {
        let content = fs::read_to_string(&json_path)?;
        serde_json::from_str(&content)?
    } else {
        let mut data = TestData::default();
        data.device_info.serial = serial.to_string();
        // 初始化测试内容所有项目为"Not started"
        data.test_content.app = "Not started".to_string();
        data.test_content.atx = "Not started".to_string();
        data.test_content.emmc = "Not started".to_string();
        data.test_content.eth = "Not started".to_string();
        data.test_content.lt6911 = "Not started".to_string();
        data.test_content.lt86102 = "Not started".to_string();
        data.test_content.rotary = "Not started".to_string();
        data.test_content.screen = "Not started".to_string();
        data.test_content.sdcard = "Not started".to_string();
        data.test_content.touch = "Not started".to_string();
        data.test_content.uart = "Not started".to_string();
        data.test_content.usb = "Not started".to_string();
        data.test_content.wifi = "Not started".to_string();
        data.test_content.ws2812 = "Not started".to_string();
        data
    };
    
    // 设置测试状态
    match item {
        // device_info项目
        "serial" => test_data.device_info.serial = status.to_string(),
        "soc_uid" => test_data.device_info.soc_uid = status.to_string(),
        "hardware" => test_data.device_info.hardware = status.to_string(),
        "wifi_exist" => test_data.device_info.wifi_exist = status.parse::<bool>()?, "test_pass" => test_data.device_info.test_pass = status.parse::<bool>()?, "unuploaded" => test_data.device_info.unuploaded = status.parse::<bool>()?,
        // test_content项目
        "app" => test_data.test_content.app = status.to_string(),
        "atx" => test_data.test_content.atx = status.to_string(),
        "emmc" => test_data.test_content.emmc = status.to_string(),
        "eth" => test_data.test_content.eth = status.to_string(),
        "lt6911" => test_data.test_content.lt6911 = status.to_string(),
        "lt86102" => test_data.test_content.lt86102 = status.to_string(),
        "rotary" => test_data.test_content.rotary = status.to_string(),
        "screen" => test_data.test_content.screen = status.to_string(),
        "sdcard" => test_data.test_content.sdcard = status.to_string(),
        "touch" => test_data.test_content.touch = status.to_string(),
        "uart" => test_data.test_content.uart = status.to_string(),
        "usb" => test_data.test_content.usb = status.to_string(),
        "wifi" => test_data.test_content.wifi = status.to_string(),
        "ws2812" => test_data.test_content.ws2812 = status.to_string(),
        _ => return Err(format!("未知的测试项目: {}", item).into()),
    }
    
    // 如果是test_pass为true，更新test_content所有项目为"Normal"
    if item == "test_pass" && status == "true" {
        test_data.test_content.app = "Normal".to_string();
        test_data.test_content.atx = "Normal".to_string();
        test_data.test_content.emmc = "Normal".to_string();
        test_data.test_content.eth = "Normal".to_string();
        test_data.test_content.lt6911 = "Normal".to_string();
        test_data.test_content.lt86102 = "Normal".to_string();
        test_data.test_content.rotary = "Normal".to_string();
        test_data.test_content.screen = "Normal".to_string();
        test_data.test_content.sdcard = "Normal".to_string();
        test_data.test_content.touch = "Normal".to_string();
        test_data.test_content.uart = "Normal".to_string();
        test_data.test_content.usb = "Normal".to_string();
        test_data.test_content.wifi = "Normal".to_string();
        test_data.test_content.ws2812 = "Normal".to_string();
    }
    
    // 保存数据到JSON文件
    let json_content = serde_json::to_string_pretty(&test_data)?;
    fs::write(&json_path, json_content)?;
    
    Ok(())
}

/// 检测应用程序根目录下的app文件夹是否为空或不存在
/// 
/// # 返回
/// - `false` 如果app文件夹存在且不为空
/// - `true` 如果app文件夹不存在或为空
pub fn is_app_folder_empty() -> bool {
    // 获取应用程序根路径
    let root_path = match get_app_root() {
        Ok(path) => path,
        Err(_) => return true,
    };
    
    // 构建app文件夹路径
    let app_folder_path = root_path.join("app");
    
    // 检查文件夹是否存在
    if !app_folder_path.exists() {
        return true;
    }
    
    // 检查是否为文件夹
    if !app_folder_path.is_dir() {
        return true;
    }
    
    // 读取文件夹内容
    match fs::read_dir(&app_folder_path) {
        Ok(entries) => {
            // 检查是否有任何文件或子文件夹
            entries.count() == 0
        }
        Err(_) => {
            // 读取失败，视为空文件夹
            true
        }
    }
}

// 获取app文件夹内tar结尾的文件路径，比如获取出来的内容如下：
// file_path = "C:\\Users\\BuGu\\AppData\\Local\\NanoKVM-Testing\\app\\NanoKVM_Pro_Testing_V2_0.tar";
pub fn get_app_file_path() -> PathBuf {
    // 获取应用程序根路径
    let root_path = match get_app_root() {
        Ok(path) => path,
        Err(_) => return PathBuf::new(),
    };
    
    // 构建app文件夹路径
    let app_dir = root_path.join("app");
    
    // 检查app文件夹是否存在
    if !app_dir.exists() {
        return PathBuf::new();
    }
    
    // 读取app文件夹内的文件
    match fs::read_dir(&app_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    
                    // 检查是否为文件且以.tar结尾
                    if path.is_file() && path.extension().map_or(false, |ext| ext == "tar") {
                        return path;
                    }
                }
            }
            
            // 如果没有找到.tar文件，返回空路径
            PathBuf::new()
        }
        Err(_) => {
            // 读取文件夹失败，返回空路径
            PathBuf::new()
        }
    }
}

/// 获取测试状态
/// 
/// # 参数
/// - `serial`: 设备序列号，作为JSON文件名的索引
/// - `item`: 测试项目名称
/// 
/// # 返回
/// - `Ok(状态字符串)` 如果获取成功
/// - `Ok(空字符串)` 如果项目不存在
/// - `Err(错误信息)` 如果获取失败
pub fn get_test_status(serial: &str, item: &str) -> Result<String, Box<dyn std::error::Error>> {
    let root_path = get_app_root()?;
    
    // 构建JSON文件路径
    let json_path = root_path.join("data").join("save").join(format!("{}.json", serial));
    
    // 检查文件是否存在
    if !json_path.exists() {
        return Ok("Not started".to_string());
    }
    
    // 读取文件内容
    let content = fs::read_to_string(&json_path)?;
    let test_data: TestData = serde_json::from_str(&content)?;
    
    // 获取测试状态
    let status = match item {
        // device_info项目
        "serial" => test_data.device_info.serial, "soc_uid" => test_data.device_info.soc_uid, "hardware" => test_data.device_info.hardware, "wifi_exist" => test_data.device_info.wifi_exist.to_string(), "test_pass" => test_data.device_info.test_pass.to_string(), "unuploaded" => test_data.device_info.unuploaded.to_string(),
        // test_content项目
        "app" => test_data.test_content.app,
        "atx" => test_data.test_content.atx,
        "emmc" => test_data.test_content.emmc,
        "eth" => test_data.test_content.eth,
        "lt6911" => test_data.test_content.lt6911,
        "lt86102" => test_data.test_content.lt86102,
        "rotary" => test_data.test_content.rotary,
        "screen" => test_data.test_content.screen,
        "sdcard" => test_data.test_content.sdcard,
        "touch" => test_data.test_content.touch,
        "uart" => test_data.test_content.uart,
        "usb" => test_data.test_content.usb,
        "wifi" => test_data.test_content.wifi,
        "ws2812" => test_data.test_content.ws2812,
        _ => return Ok("Not started".to_string()),
    };
    
    Ok(status)
}

/// 设置测试日志
/// 
/// # 参数
/// - `serial`: 设备序列号，作为JSON文件名的索引
/// - `date`: 日期，格式为"YYYY-MM-DD"
/// - `item`: 测试项目名称
/// - `log`: 测试日志内容
/// 
/// # 返回
/// - `Ok(())` 如果设置成功
/// - `Err(错误信息)` 如果设置失败
pub fn set_test_log(serial: &str, date: &str, item: &str, log: &str) -> Result<(), Box<dyn std::error::Error>> {
    let root_path = get_app_root()?;
    
    // 构建JSON文件路径
    let json_path = root_path.join("data").join("save").join(format!("{}.json", serial));
    
    // 读取现有数据或创建新数据
    let mut test_data = if json_path.exists() {
        let content = fs::read_to_string(&json_path)?;
        serde_json::from_str(&content)?
    } else {
        let mut data = TestData::default();
        data.device_info.serial = serial.to_string();
        data
    };
    
    // 获取或创建指定日期的日志条目
    let log_entry = test_data.test_log.entries.entry(date.to_string()).or_insert(TestLogEntry::default());
    
    // 设置日志内容
    if item == "test_pass" {
        // 处理test_pass字段，转换为bool类型
        log_entry.test_pass = log.parse::<bool>()?;
    } else {
        // 处理其他字段，添加到other_fields哈希表
        log_entry.other_fields.insert(item.to_string(), log.to_string());
    }
    
    // 保存数据到JSON文件
    let json_content = serde_json::to_string_pretty(&test_data)?;
    fs::write(&json_path, json_content)?;
    
    Ok(())
}

/// 创建新的串号，根据日期，测试主机编号，已经存储的数量等生成新的编号，规则如下
/// 串号规则：
// N d a L 0 0 0 0 0
// │ │ │ │ │ │
// │ │ │ │ │ └─ 序列号，十六进制
// │ │ │ │ └─── 测试主机
// │ │ │ └───── 周代码
// │ │ └─────── 年代码
// │ └───────── 产品配置/子类
// └─────────── 产品代号
// 产品代号
//     N: NanoKVM
// 产品配置/子类
//     a: NanoKVM-Alpha
//     b: NanoKVM-Beta
//     c: NanoKVM-PCIe
//     d: NanoKVM-Pro-ATX-Alpha
//     e: NanoKVM-Pro-Desk-Alpha
// 年代码
//     a: 2025
//     b: 2026
//     ……
// 周代码(第？周)
//     a-z: 1-26周
//     A-Z: 27-52周
// 测试主机
//     0：产测V1主机（618产测）
//     1-9：产测V2主机（x86）
///// 产品代号（4位十六进制，前面几位相同时产品代号从0递增）
pub fn create_serial_number(product_config: &str) -> Result<String, Box<dyn std::error::Error>> {
    // 获取当前日期
    let now = Local::now();
    let year = now.year();
    let week_number = now.iso_week().week() as u8;
    
    // 1. 产品代号 (固定为N)
    let product_code = "N";
    
    // 2. 产品配置/子类
    let mut config_code = "e";      // default to Desk
    if product_config.contains("ATX") {
        config_code = "d";
    } 
    
    // 3. 年代码 (a=2025, b=2026, ...)
    let year_code = ((year - 2025) as u8 + b'a') as char;
    
    // 4. 周代码 (a-z: 1-26周, A-Z: 27-52周)
    let week_code = if week_number <= 26 {
        (week_number - 1 + b'a') as char
    } else {
        (week_number - 27 + b'A') as char
    };
    
    // 5. 测试主机编号
    let machine_number = get_config_str("application", "machine_number")
        .unwrap_or("1".to_string());
    
    // 6. 序列号 (5位十六进制，前面几位相同时从0递增)
    let root_path = get_app_root()?;
    let save_path = root_path.join("data").join("save");
    
    // 检查save目录是否存在
    if !save_path.exists() {
        fs::create_dir_all(&save_path)?;
    }
    
    // 统计与当前前缀相同的序列号数量
    let prefix = format!("{}{}{}{}{}", product_code, config_code, year_code, week_code, machine_number);
    let mut serial_count = 0;
    
    for entry in fs::read_dir(&save_path)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        
        // 检查文件是否为JSON文件且以当前前缀开头
        if file_name.ends_with(".json") && file_name.starts_with(&prefix) {
            serial_count += 1;
        }
    }
    
    // 生成4位十六进制序列号
    let serial_hex = format!("{:04X}", serial_count);
    
    // 组合所有部分生成完整序列号
    let serial_number = format!("{}{}{}{}{}{}", product_code, config_code, year_code, week_code, machine_number, serial_hex);
    
    Ok(serial_number)
} 