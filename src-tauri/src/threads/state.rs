use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use tauri::{AppHandle, Emitter};
use std::time::Duration;

// 大状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainState {
    /// 状态1: 初始状态
    State1,
    /// 状态2: 中间状态
    State2,
    /// 状态3: 最终状态
    State3,
}

// 状态1的小状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubState1 {
    /// 步骤1: 初始化
    Step1,
    /// 步骤2: 准备
    Step2,
    /// 步骤3: 就绪
    Step3,
}

// 状态2的小状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubState2 {
    /// 步骤1: 连接
    Step1,
    /// 步骤2: 验证
    Step2,
    /// 步骤3: 配置
    Step3,
    /// 步骤4: 运行
    Step4,
}

// 状态3的小状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubState3 {
    /// 步骤1: 监控
    Step1,
    /// 步骤2: 优化
    Step2,
    /// 步骤3: 完成
    Step3,
}

// 并行状态标志
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParallelFlags {
    /// 并行任务1是否完成
    pub task1_completed: bool,
    /// 并行任务2是否完成
    pub task2_completed: bool,
    /// 并行任务3是否完成
    pub task3_completed: bool,
}

impl Default for ParallelFlags {
    fn default() -> Self {
        Self {
            task1_completed: false,
            task2_completed: false,
            task3_completed: false,
        }
    }
}

// 全局状态结构
#[derive(Debug, Clone)]
pub struct GlobalState {
    /// 当前大状态
    pub main_state: MainState,
    /// 状态1的小状态
    pub sub_state1: Option<SubState1>,
    /// 状态2的小状态
    pub sub_state2: Option<SubState2>,
    /// 状态3的小状态
    pub sub_state3: Option<SubState3>,
    /// 并行状态标志
    pub parallel_flags: ParallelFlags,
    /// 是否处于断开状态（用于快速重置）
    pub disconnected: bool,
    /// 状态变更时间戳
    pub last_change_time: std::time::Instant,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            main_state: MainState::State1,
            sub_state1: Some(SubState1::Step1),
            sub_state2: None,
            sub_state3: None,
            parallel_flags: ParallelFlags::default(),
            disconnected: false,
            last_change_time: std::time::Instant::now(),
        }
    }
}

// 状态管理器
pub struct StateManager {
    state: Arc<Mutex<GlobalState>>,
    app_handle: Option<AppHandle>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(GlobalState::default())),
            app_handle: None,
        }
    }

    /// 设置应用句柄用于事件通知
    pub fn set_app_handle(&mut self, app_handle: AppHandle) {
        self.app_handle = Some(app_handle);
    }

    /// 获取当前状态（只读）
    pub fn get_state(&self) -> GlobalState {
        self.state.lock().unwrap().clone()
    }

    /// 设置大状态
    pub fn set_main_state(&self, new_state: MainState) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        
        // 检查状态转换是否合法
        if !Self::is_valid_transition(state.main_state, new_state) {
            return Err(format!("无效的状态转换: {:?} -> {:?}", state.main_state, new_state));
        }

        state.main_state = new_state;
        state.last_change_time = std::time::Instant::now();
        
        // 根据新的大状态重置小状态
        match new_state {
            MainState::State1 => {
                state.sub_state1 = Some(SubState1::Step1);
                state.sub_state2 = None;
                state.sub_state3 = None;
                state.parallel_flags = ParallelFlags::default();
            }
            MainState::State2 => {
                state.sub_state2 = Some(SubState2::Step1);
                state.sub_state3 = None;
                state.parallel_flags = ParallelFlags::default();
            }
            MainState::State3 => {
                state.sub_state3 = Some(SubState3::Step1);
                state.parallel_flags = ParallelFlags::default();
            }
        }

        // 发送状态变更事件
        self.emit_state_change(&state);
        Ok(())
    }

    /// 推进小状态（在当前大状态下）
    pub fn advance_sub_state(&self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        
        match state.main_state {
            MainState::State1 => {
                if let Some(current) = state.sub_state1 {
                    state.sub_state1 = match current {
                        SubState1::Step1 => Some(SubState1::Step2),
                        SubState1::Step2 => Some(SubState1::Step3),
                        SubState1::Step3 => {
                            // 状态1完成，自动进入状态2
                            state.main_state = MainState::State2;
                            state.sub_state2 = Some(SubState2::Step1);
                            state.sub_state1 = None;
                            None
                        }
                    };
                }
            }
            MainState::State2 => {
                if let Some(current) = state.sub_state2 {
                    state.sub_state2 = match current {
                        SubState2::Step1 => Some(SubState2::Step2),
                        SubState2::Step2 => Some(SubState2::Step3),
                        SubState2::Step3 => Some(SubState2::Step4),
                        SubState2::Step4 => {
                            // 状态2完成，自动进入状态3
                            state.main_state = MainState::State3;
                            state.sub_state3 = Some(SubState3::Step1);
                            state.sub_state2 = None;
                            None
                        }
                    };
                }
            }
            MainState::State3 => {
                if let Some(current) = state.sub_state3 {
                    state.sub_state3 = match current {
                        SubState3::Step1 => Some(SubState3::Step2),
                        SubState3::Step2 => Some(SubState3::Step3),
                        SubState3::Step3 => {
                            // 状态3完成，保持最终状态
                            Some(SubState3::Step3)
                        }
                    };
                }
            }
        }

        state.last_change_time = std::time::Instant::now();
        self.emit_state_change(&state);
        Ok(())
    }

    /// 设置并行任务完成状态
    pub fn set_parallel_task_completed(&self, task_id: u32) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();
        
        match task_id {
            1 => state.parallel_flags.task1_completed = true,
            2 => state.parallel_flags.task2_completed = true,
            3 => state.parallel_flags.task3_completed = true,
            _ => return Err(format!("无效的任务ID: {}", task_id)),
        }

        state.last_change_time = std::time::Instant::now();
        self.emit_state_change(&state);
        Ok(())
    }

    /// 检查所有并行任务是否完成
    pub fn are_all_parallel_tasks_completed(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.parallel_flags.task1_completed 
            && state.parallel_flags.task2_completed 
            && state.parallel_flags.task3_completed
    }

    /// 处理断开事件 - 立即重置到状态1的第一步
    pub fn handle_disconnect(&self) {
        let mut state = self.state.lock().unwrap();
        
        state.main_state = MainState::State1;
        state.sub_state1 = Some(SubState1::Step1);
        state.sub_state2 = None;
        state.sub_state3 = None;
        state.parallel_flags = ParallelFlags::default();
        state.disconnected = true;
        state.last_change_time = std::time::Instant::now();

        // 发送断开事件
        self.emit_state_change(&state);
        
        // 发送断开通知
        if let Some(app_handle) = &self.app_handle {
            let _ = app_handle.emit("state-disconnected", serde_json::json!({}));
        }
    }

    /// 清除断开标志
    pub fn clear_disconnect_flag(&self) {
        let mut state = self.state.lock().unwrap();
        state.disconnected = false;
        state.last_change_time = std::time::Instant::now();
        self.emit_state_change(&state);
    }

    /// 检查状态转换是否合法
    fn is_valid_transition(from: MainState, to: MainState) -> bool {
        match (from, to) {
            (MainState::State1, MainState::State2) => true,
            (MainState::State2, MainState::State3) => true,
            (MainState::State1, MainState::State1) => true, // 允许重置
            (MainState::State2, MainState::State1) => true, // 允许回退（断开时）
            (MainState::State3, MainState::State1) => true, // 允许回退（断开时）
            _ => false,
        }
    }

    /// 发送状态变更事件
    fn emit_state_change(&self, state: &GlobalState) {
        if let Some(app_handle) = &self.app_handle {
            let _ = app_handle.emit("state-changed", serde_json::json!({
                "main_state": format!("{:?}", state.main_state),
                "sub_state1": state.sub_state1.map(|s| format!("{:?}", s)),
                "sub_state2": state.sub_state2.map(|s| format!("{:?}", s)),
                "sub_state3": state.sub_state3.map(|s| format!("{:?}", s)),
                "disconnected": state.disconnected,
                "parallel_flags": {
                    "task1_completed": state.parallel_flags.task1_completed,
                    "task2_completed": state.parallel_flags.task2_completed,
                    "task3_completed": state.parallel_flags.task3_completed,
                }
            }));
        }
    }

    /// 获取状态持续时间
    pub fn get_state_duration(&self) -> Duration {
        let state = self.state.lock().unwrap();
        state.last_change_time.elapsed()
    }
}

// 全局状态管理器实例
lazy_static! {
    pub static ref GLOBAL_STATE_MANAGER: StateManager = StateManager::new();
}

// 公共API函数

/// 初始化状态管理器
pub fn init_state_manager(app_handle: AppHandle) {
    GLOBAL_STATE_MANAGER.set_app_handle(app_handle);
}

/// 获取当前状态
pub fn get_current_state() -> GlobalState {
    GLOBAL_STATE_MANAGER.get_state()
}

/// 设置大状态
pub fn set_main_state(new_state: MainState) -> Result<(), String> {
    GLOBAL_STATE_MANAGER.set_main_state(new_state)
}

/// 推进小状态
pub fn advance_sub_state() -> Result<(), String> {
    GLOBAL_STATE_MANAGER.advance_sub_state()
}

/// 设置并行任务完成
pub fn set_parallel_task_completed(task_id: u32) -> Result<(), String> {
    GLOBAL_STATE_MANAGER.set_parallel_task_completed(task_id)
}

/// 处理断开事件
pub fn handle_disconnect() {
    GLOBAL_STATE_MANAGER.handle_disconnect()
}

/// 清除断开标志
pub fn clear_disconnect_flag() {
    GLOBAL_STATE_MANAGER.clear_disconnect_flag()
}

/// 检查所有并行任务是否完成
pub fn are_all_parallel_tasks_completed() -> bool {
    GLOBAL_STATE_MANAGER.are_all_parallel_tasks_completed()
}

/// 获取状态持续时间
pub fn get_state_duration() -> Duration {
    GLOBAL_STATE_MANAGER.get_state_duration()
}