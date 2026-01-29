import { TestButton } from './TestButton';
import { useEffect, useState } from 'react';
import { TestButtonStatus, TestButtonId } from '../services/backendService';

interface SecondTestPanelProps {
  isDark: boolean;
}

// 后测按钮状态管理
interface SecondTestButtonStates {
  wait_connection: TestButtonStatus;
  wait_power_on: TestButtonStatus;
  get_status: TestButtonStatus;
  video_capture: TestButtonStatus;
  wifi_exist: TestButtonStatus;
  touch: TestButtonStatus;
  knob: TestButtonStatus;
  print_label: TestButtonStatus;
}

export function SecondTestPanel({ isDark }: SecondTestPanelProps) {
  // 初始化所有按钮状态为未测试
  const [buttonStates, setButtonStates] = useState<SecondTestButtonStates>({
    wait_connection: 'untested',
    wait_power_on: 'untested',
    get_status: 'untested',
    video_capture: 'untested',
    wifi_exist: 'untested',
    touch: 'untested',
    knob: 'untested',
    print_label: 'untested',
  });

  // 监听测试按钮状态更新事件
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    
    const setupListener = async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        unlisten = await listen('second-test-button-status-update', (event) => {
          const { buttonId, status } = event.payload as { buttonId: TestButtonId; status: TestButtonStatus };
          setButtonStates(prev => ({
            ...prev,
            [buttonId]: status
          }));
        });
      } catch (error) {
        console.error('监听后测按钮状态更新失败:', error);
      }
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);
  
  return (
    <main className="flex-1 p-8 overflow-auto">
      {/* 初始化 */}
      <section className="mb-8">
        <h2 className={`mb-4 ${isDark ? 'text-white' : 'text-neutral-900'}`} style={{ fontSize: '24px', fontWeight: 'bold' }}>初始化</h2>
        <div className="flex flex-wrap gap-3">
          <TestButton status={buttonStates.wait_connection} isDark={isDark}>等待连接</TestButton>
          <TestButton status={buttonStates.wait_power_on} isDark={isDark}>等待开机</TestButton>
          <TestButton status={buttonStates.get_status} isDark={isDark}>获取状态</TestButton>
        </div>
      </section>

      <hr className={`mb-8 ${isDark ? 'border-neutral-700' : 'border-neutral-300'}`} />

      {/* 项目测试 */}
      <section className="mb-8">
        <h2 className={`mb-4 ${isDark ? 'text-white' : 'text-neutral-900'}`} style={{ fontSize: '24px', fontWeight: 'bold' }}>接口测试</h2>
        
        <div className="flex flex-wrap gap-3 mb-3">
          <div className="w-24 flex items-center">
            <span className={isDark ? 'text-white' : 'text-neutral-900'}>硬件</span>
          </div>
          <TestButton status={buttonStates.video_capture} isDark={isDark}>HDMI捕获</TestButton>
          <TestButton status={buttonStates.wifi_exist} isDark={isDark}>WiFi</TestButton>
        </div>

        <div className="flex flex-wrap gap-3 mb-3">
          <div className="w-24 flex items-center">
            <span className={isDark ? 'text-white' : 'text-neutral-900'}>交互</span>
          </div>
          <TestButton status={buttonStates.touch} isDark={isDark}>屏幕触摸</TestButton>
          <TestButton status={buttonStates.knob} isDark={isDark}>旋钮测试</TestButton>
        </div>
      </section>

      <hr className={`mb-8 ${isDark ? 'border-neutral-700' : 'border-neutral-300'}`} />

      {/* 信息打印 */}
      <section className="mb-8">
        <h2 className={`mb-4 ${isDark ? 'text-white' : 'text-neutral-900'}`} style={{ fontSize: '24px', fontWeight: 'bold' }}>信息打印</h2>
        <div className="flex flex-wrap gap-3">
          <TestButton status={buttonStates.print_label} isDark={isDark}>标签打印</TestButton>
        </div>
      </section>
    </main>
  );
}