// 后端服务 - 与 Rust 后端通信

// 扩展 Window 类型以包含 __TAURI__
declare global {
  interface Window {
    __TAURI__?: any;
  }
}

// 开发环境检测
const isDevEnvironment = typeof window !== 'undefined' && !window.__TAURI__;

// 获取侧边栏信息
export async function getSidebarInfo(): Promise<{
  machineCode: string;
  serverStatus: string;
  currentDevice: string;
  serialNumber: string;
  targetIP: string;
  uploadCount: number;
}> {
  if (isDevEnvironment) {
    // 开发环境返回模拟数据
    return {
      machineCode: "DEV-001",
      serverStatus: "online",
      currentDevice: "Desk-A",
      serialNumber: "Neal0015B",
      targetIP: "192.168.222.222",
      uploadCount: 32
    };
  }

  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const result = await invoke('get_sidebar_info');
    return result as {
      machineCode: string;
      serverStatus: string;
      currentDevice: string;
      serialNumber: string;
      targetIP: string;
      uploadCount: number;
    };
  } catch (error) {
    console.error('获取侧边栏信息失败:', error);
    throw error;
  }
}

// 设置机器码
export async function setMachineCode(code: string): Promise<void> {
  if (isDevEnvironment) {
    console.log('开发环境: 设置机器码为', code);
    return;
  }

  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('set_machine_code', { code });
  } catch (error) {
    console.error('设置机器码失败:', error);
    throw error;
  }
}

// 设置服务器状态
export async function setServerStatus(status: string): Promise<void> {
  if (isDevEnvironment) {
    console.log('开发环境: 设置服务器状态为', status);
    return;
  }

  try {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('set_server_status', { status });
  } catch (error) {
    console.error('设置服务器状态失败:', error);
    throw error;
  }
}