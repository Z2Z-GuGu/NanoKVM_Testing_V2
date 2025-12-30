use windows::{
    core::Result,
    Win32::{
        Foundation::{BOOL, LPARAM, RECT},
        Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR},
    },
};

// 回调函数，用于处理每个显示器
unsafe extern "system" fn monitor_enum_proc(
    _hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    // 将 lparam 转换回计数器指针
    let count = unsafe { &mut *(lparam.0 as *mut u32) };
    *count += 1;
    BOOL::from(true) // 继续枚举
}

fn get_monitor_count() -> Result<u32> {
    unsafe {
        let mut count = 0;
        
        // 枚举所有显示器
        EnumDisplayMonitors(
            None,
            None,
            Some(monitor_enum_proc),
            LPARAM(&mut count as *mut _ as isize),
        )
        .ok()?;
        
        Ok(count)
    }
}

pub fn if_two_monitor() -> bool {
    let count = get_monitor_count().unwrap_or_else(|e| {
        eprintln!("枚举显示器失败: {:?}", e);
        0
    });
    count >= 2
}

// fn main() -> Result<()> {
//     let monitors = enumerate_monitors()?;
//     
//     println!("检测到 {} 个显示器：", monitors.len());
//     println!("{}", "=".repeat(50));
//     
//     for (i, monitor) in monitors.iter().enumerate() {
//         println!("显示器 #{}:", i + 1);
//         println!("  设备名称: {}", monitor.device_name);
//         println!("  分辨率: {} × {}", monitor.width(), monitor.height());
//         println!("  位置: ({}, {})", monitor.left, monitor.top);
//         println!("  HMONITOR: {:?}", monitor.handle);
//         println!();
//     }
//     
//     Ok(())
// }