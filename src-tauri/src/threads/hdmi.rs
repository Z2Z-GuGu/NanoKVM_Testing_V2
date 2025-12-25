use windows::{
    core::Result,
    Win32::{
        Foundation::{BOOL, LPARAM, RECT},
        Graphics::Gdi::{
            EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR,
            MONITORINFOEXW, 
        },
    },
};

// 回调函数，用于处理每个显示器
unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    // 将 lparam 转换回我们的显示器列表
    let monitors = unsafe { &mut *(lparam.0 as *mut Vec<MonitorInfo>) };
    
    // 获取显示器信息
    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
    
    if unsafe { GetMonitorInfoW(hmonitor, &mut info as *mut _ as *mut _) }.as_bool() {
        let monitor_info = MonitorInfo {
            handle: hmonitor,
            left: info.monitorInfo.rcMonitor.left,
            top: info.monitorInfo.rcMonitor.top,
            right: info.monitorInfo.rcMonitor.right,
            bottom: info.monitorInfo.rcMonitor.bottom,
            device_name: String::from_utf16_lossy(
                &info.szDevice[0..].iter()
                    .take_while(|&&c| c != 0)
                    .map(|&c| c as u16)
                    .collect::<Vec<u16>>()
            ),
        };
        monitors.push(monitor_info);
    }
    
    BOOL::from(true) // 继续枚举
}

// 显示器信息结构体
#[derive(Debug)]
struct MonitorInfo {
    handle: HMONITOR,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    device_name: String,
}

// impl MonitorInfo {
//     fn width(&self) -> i32 {
//         self.right - self.left
//     }
    
//     fn height(&self) -> i32 {
//         self.bottom - self.top
//     }
// }

fn enumerate_monitors() -> Result<Vec<MonitorInfo>> {
    unsafe {
        let mut monitors = Vec::new();
        
        // 枚举所有显示器
        EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum_proc),
            LPARAM(&mut monitors as *mut _ as isize),
        )
        .ok()?;
        
        Ok(monitors)
    }
}

pub fn if_two_monitor() -> bool {
    let monitors = enumerate_monitors().unwrap_or_else(|e| {
        eprintln!("枚举显示器失败: {:?}", e);
        Vec::new()
    });
    monitors.len() >= 2
}

// fn main() -> Result<()> {
//     let monitors = enumerate_monitors()?;
    
//     println!("检测到 {} 个显示器：", monitors.len());
//     println!("{}", "=".repeat(50));
    
//     for (i, monitor) in monitors.iter().enumerate() {
//         println!("显示器 #{}:", i + 1);
//         println!("  设备名称: {}", monitor.device_name);
//         println!("  分辨率: {} × {}", monitor.width(), monitor.height());
//         println!("  位置: ({}, {})", monitor.left, monitor.top);
//         println!("  HMONITOR: {:?}", monitor.handle);
//         println!();
//     }
    
//     Ok(())
// }