import { TestButton } from './TestButton';
import { useEffect, useState } from 'react';

import { TestButtonStatus, TestButtonId } from '../services/backendService';

interface TestPanelProps {
  isDark: boolean;
}

// 测试按钮状态管理
interface TestButtonStates {
  wait_connection: TestButtonStatus;
  wait_boot: TestButtonStatus;
  get_ip: TestButtonStatus;
  download_test: TestButtonStatus;
  detect_hardware: TestButtonStatus;
  emmc_test: TestButtonStatus;
  dtb: TestButtonStatus;
  uboot: TestButtonStatus;
  kernel: TestButtonStatus;
  app_install: TestButtonStatus;
  hdmi_wait_connection: TestButtonStatus;
  hdmi_io_test: TestButtonStatus;
  hdmi_loop_test: TestButtonStatus;
  hdmi_capture_test: TestButtonStatus;
  hdmi_version: TestButtonStatus;
  hdmi_write_edid: TestButtonStatus;
  usb_wait_connection: TestButtonStatus;
  eth_wait_connection: TestButtonStatus;
  eth_upload_test: TestButtonStatus;
  eth_download_test: TestButtonStatus;
  wifi_wait_connection: TestButtonStatus;
  wifi_upload_test: TestButtonStatus;
  wifi_download_test: TestButtonStatus;
  screen: TestButtonStatus;
  touch: TestButtonStatus;
  knob: TestButtonStatus;
  atx: TestButtonStatus;
  io: TestButtonStatus;
  tf_card: TestButtonStatus;
  uart: TestButtonStatus;
  auto_start: TestButtonStatus;
}

export function TestPanel({ isDark }: TestPanelProps) {
  // 初始化所有按钮状态为未测试
  const [buttonStates, setButtonStates] = useState<TestButtonStates>({
    wait_connection: 'untested',
    wait_boot: 'untested',
    get_ip: 'untested',
    download_test: 'untested',
    detect_hardware: 'untested',
    emmc_test: 'untested',
    dtb: 'untested',
    uboot: 'untested',
    kernel: 'untested',
    app_install: 'untested',
    hdmi_wait_connection: 'untested',
    hdmi_io_test: 'untested',
    hdmi_loop_test: 'untested',
    hdmi_capture_test: 'untested',
    hdmi_version: 'untested',
    hdmi_write_edid: 'untested',
    usb_wait_connection: 'untested',
    eth_wait_connection: 'untested',
    eth_upload_test: 'untested',
    eth_download_test: 'untested',
    wifi_wait_connection: 'untested',
    wifi_upload_test: 'untested',
    wifi_download_test: 'untested',
    screen: 'untested',
    touch: 'untested',
    knob: 'untested',
    atx: 'untested',
    io: 'untested',
    tf_card: 'untested',
    uart: 'untested',
    auto_start: 'untested',
  });

  // 监听测试按钮状态更新事件
  useEffect(() => {
    let unlisten: (() => void) | null = null;
    
    const setupListener = async () => {
      try {
        const { listen } = await import('@tauri-apps/api/event');
        unlisten = await listen('test-button-status-update', (event) => {
          const { buttonId, status } = event.payload as { buttonId: TestButtonId; status: TestButtonStatus };
          setButtonStates(prev => ({
            ...prev,
            [buttonId]: status
          }));
        });
      } catch (error) {
        console.error('监听测试按钮状态更新失败:', error);
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
          <TestButton status={buttonStates.wait_boot} isDark={isDark}>等待开机</TestButton>
          <TestButton status={buttonStates.get_ip} isDark={isDark}>获取IP</TestButton>
          <TestButton status={buttonStates.download_test} isDark={isDark}>下载产测</TestButton>
          <TestButton status={buttonStates.detect_hardware} isDark={isDark}>检测硬件</TestButton>
          <TestButton status={buttonStates.emmc_test} isDark={isDark}>eMMC</TestButton>
        </div>
      </section>

      <hr className={`mb-8 ${isDark ? 'border-neutral-700' : 'border-neutral-300'}`} />

      {/* 项目测试 */}
      <section className="mb-8">
        <h2 className={`mb-4 ${isDark ? 'text-white' : 'text-neutral-900'}`} style={{ fontSize: '24px', fontWeight: 'bold' }}>项目测试</h2>
        
        {/* 系统更新 */}
        <div className="flex flex-wrap gap-3 mb-3">
          <div className="w-24 flex items-center">
            <span className={isDark ? 'text-white' : 'text-neutral-900'}>系统更新</span>
          </div>
          <TestButton status={buttonStates.dtb} isDark={isDark}>dtb</TestButton>
          <TestButton status={buttonStates.uboot} isDark={isDark}>uboot</TestButton>
          <TestButton status={buttonStates.kernel} isDark={isDark}>kernel</TestButton>
          <TestButton status={buttonStates.app_install} isDark={isDark}>APP安装</TestButton>
        </div>

        {/* HDMI接口 */}
        <div className="flex flex-wrap gap-3 mb-3">
          <div className="w-24 flex items-center">
            <span className={isDark ? 'text-white' : 'text-neutral-900'}>HDMI接口</span>
          </div>
          <TestButton status={buttonStates.hdmi_wait_connection} isDark={isDark}>等待连接</TestButton>
          <TestButton status={buttonStates.hdmi_io_test} isDark={isDark}>测试IO</TestButton>
          <TestButton status={buttonStates.hdmi_loop_test} isDark={isDark}>测试环出</TestButton>
          <TestButton status={buttonStates.hdmi_capture_test} isDark={isDark}>测试采集</TestButton>
          <TestButton status={buttonStates.hdmi_version} isDark={isDark}>Version</TestButton>
          <TestButton status={buttonStates.hdmi_write_edid} isDark={isDark}>写EDID</TestButton>
        </div>

        {/* USB接口 */}
        <div className="flex flex-wrap gap-3 mb-3">
          <div className="w-24 flex items-center">
            <span className={isDark ? 'text-white' : 'text-neutral-900'}>USB接口</span>
          </div>
          <TestButton status={buttonStates.usb_wait_connection} isDark={isDark}>等待连接</TestButton>
        </div>

        {/* ETH接口 */}
        <div className="flex flex-wrap gap-3 mb-3">
          <div className="w-24 flex items-center">
            <span className={isDark ? 'text-white' : 'text-neutral-900'}>ETH接口</span>
          </div>
          <TestButton status={buttonStates.eth_wait_connection} isDark={isDark}>等待连接</TestButton>
          <TestButton status={buttonStates.eth_upload_test} isDark={isDark}>上传测速</TestButton>
          <TestButton status={buttonStates.eth_download_test} isDark={isDark}>下载测速</TestButton>
        </div>

        {/* WiFi */}
        <div className="flex flex-wrap gap-3 mb-3">
          <div className="w-24 flex items-center">
            <span className={isDark ? 'text-white' : 'text-neutral-900'}>WiFi</span>
          </div>
          <TestButton status={buttonStates.wifi_wait_connection} isDark={isDark}>等待连接</TestButton>
          <TestButton status={buttonStates.wifi_upload_test} isDark={isDark}>上传测速</TestButton>
          <TestButton status={buttonStates.wifi_download_test} isDark={isDark}>下载测速</TestButton>
        </div>

        {/* 交互 */}
        <div className="flex flex-wrap gap-3 mb-3">
          <div className="w-24 flex items-center">
            <span className={isDark ? 'text-white' : 'text-neutral-900'}>交互</span>
          </div>
          <TestButton status={buttonStates.screen} isDark={isDark}>屏幕</TestButton>
          <TestButton status={buttonStates.touch} isDark={isDark}>触摸</TestButton>
          <TestButton status={buttonStates.knob} isDark={isDark}>旋钮</TestButton>
        </div>

        {/* 其他 */}
        <div className="flex flex-wrap gap-3 mb-3">
          <div className="w-24 flex items-center">
            <span className={isDark ? 'text-white' : 'text-neutral-900'}>其他</span>
          </div>
          <TestButton status={buttonStates.atx} isDark={isDark}>ATX</TestButton>
          <TestButton status={buttonStates.io} isDark={isDark}>IO</TestButton>
          <TestButton status={buttonStates.tf_card} isDark={isDark}>TF卡</TestButton>
          <TestButton status={buttonStates.uart} isDark={isDark}>UART</TestButton>
        </div>
      </section>

      <hr className={`mb-8 ${isDark ? 'border-neutral-700' : 'border-neutral-300'}`} />

      {/* 应用设置 */}
      <section>
        <h2 className={`mb-4 ${isDark ? 'text-white' : 'text-neutral-900'}`} style={{ fontSize: '24px', fontWeight: 'bold' }}>应用设置</h2>
        <div className="flex gap-3">
          <TestButton status={buttonStates.auto_start} isDark={isDark}>开机自启</TestButton>
        </div>
      </section>
    </main>
  );
}