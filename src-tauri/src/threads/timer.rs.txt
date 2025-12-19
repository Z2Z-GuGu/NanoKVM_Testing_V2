use tokio::sync::{Mutex};
use tokio::time::{sleep, Duration};
use tauri::async_runtime::spawn;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = false;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[timer]{}", msg);
    }
}

// 定时器句柄类型
pub type TimerHandle = usize;

// 定时器数据结构
struct Timer {
    current_value: u64,      // 当前计数值
    max_value: u64,          // 最大值
    is_active: bool,         // 是否激活
}

// 全局定时器管理器
lazy_static! {
    static ref TIMER_COUNTER: AtomicUsize = AtomicUsize::new(0);
    static ref TIMERS: Mutex<HashMap<TimerHandle, Timer>> = Mutex::new(HashMap::new());
}

// 定时器管理线程
async fn timer_management_task() {
    log("定时器管理线程已启动");
    
    loop {
        // 休眠1秒
        sleep(Duration::from_secs(1)).await;
        
        // 更新所有激活的定时器
        let mut timers_guard = TIMERS.lock().await;
        for (_, timer) in timers_guard.iter_mut() {
            if timer.is_active && timer.current_value < timer.max_value {
                timer.current_value += 1;
            }
        }
    }
}

// 创建定时器（最大值）
pub async fn create_timer(max_value: u64) -> TimerHandle {
    let handle = TIMER_COUNTER.fetch_add(1, Ordering::SeqCst);
    
    let mut timers_guard = TIMERS.lock().await;
    timers_guard.insert(handle, Timer {
        current_value: 0,
        max_value,
        is_active: true,
    });
    
    log(&format!("创建定时器，句柄: {}, 最大值: {}", handle, max_value));
    handle
}

// 查看定时器当前值
pub async fn check_timer(handle: TimerHandle) -> Option<u64> {
    let timers_guard = TIMERS.lock().await;
    timers_guard.get(&handle).map(|timer| timer.current_value)
}

// 归零定时器
pub async fn reset_timer(handle: TimerHandle) -> bool {
    let mut timers_guard = TIMERS.lock().await;
    if let Some(timer) = timers_guard.get_mut(&handle) {
        timer.current_value = 0;
        log(&format!("定时器 {} 已归零", handle));
        true
    } else {
        log(&format!("未找到定时器 {}", handle));
        false
    }
}

// 删除定时器
pub async fn delete_timer(handle: TimerHandle) -> bool {
    let mut timers_guard = TIMERS.lock().await;
    if timers_guard.remove(&handle).is_some() {
        log(&format!("定时器 {} 已删除", handle));
        true
    } else {
        log(&format!("未找到定时器 {}", handle));
        false
    }
}

// 启动定时器线程
pub fn spawn_timer_task() {
    spawn(async move {
        timer_management_task().await;
    });
}
