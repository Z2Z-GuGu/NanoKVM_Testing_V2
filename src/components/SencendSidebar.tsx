import { useState, useEffect } from 'react';
import { ChevronDown } from 'lucide-react';
import { listen } from '@tauri-apps/api/event';

interface SidebarProps {
  theme: '明亮' | '暗黑';
  onThemeChange: (theme: '明亮' | '暗黑') => void;
  isDark: boolean;
}

export function SecondSidebar({
  theme,
  onThemeChange,
  isDark
}: SidebarProps) {
  const [currentTime, setCurrentTime] = useState('');
  const [currentDevice, setCurrentDevice] = useState('-');
  const [serialNumber, setSerialNumber] = useState('-');

  useEffect(() => {
    const updateTime = () => {
      const now = new Date();
      const formatted = `${now.getFullYear()}年${String(now.getMonth() + 1).padStart(2, '0')}月${String(now.getDate()).padStart(2, '0')}日 - ${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
      setCurrentTime(formatted);
    };

    updateTime();
    const interval = setInterval(updateTime, 1000);

    return () => clearInterval(interval);
  }, []);

  // 监听后端的各个状态更新事件
  useEffect(() => {

    // 监听当前硬件更新
    listen('current-device-update', (event) => {
      const device = event.payload as string;
      console.log('收到当前硬件更新:', device);
      setCurrentDevice(device);
    }).catch(error => {
      console.error('设置当前硬件监听器失败:', error);
    });

    // 监听序列号更新
    listen('serial-number-update', (event) => {
      const serial = event.payload as string;
      console.log('收到序列号更新:', serial);
      setSerialNumber(serial);
    }).catch(error => {
      console.error('设置序列号监听器失败:', error);
    });
  }, []);

  const toggleTheme = () => {
    onThemeChange(theme === '明亮' ? '暗黑' : '明亮');
  };

  return (
    <aside className={`w-[300px] ${isDark ? 'bg-neutral-800' : 'bg-neutral-300'} p-5 flex flex-col justify-between`}>
      <div>
        <div className={'mb-16 mt-1'}>
          <div className="flex flex-col">
            <h1 className={`${isDark ? 'text-white' : 'text-neutral-900'}`} style={{ fontSize: '34px', fontWeight: 'bold', marginLeft: '4px', marginBottom: '16px' }}>NanoKVM-Pro</h1>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'baseline' }}>
              <h1 className={`${isDark ? 'text-white' : 'text-neutral-900'}`} style={{ fontSize: '34px', fontWeight: 'bold', lineHeight: '1.2', margin: 0, marginLeft: '4px' }}>后测工具</h1>
              <p className={'text-sm ' + (isDark ? 'text-neutral-400' : 'text-neutral-700')} style={{ fontSize: '20px', fontWeight: 'bold', margin: 0, verticalAlign: 'baseline', marginRight: '6px' }}>V2.2</p>
            </div>
          </div>
        </div>
      </div>

      <div>
        <div className={`${isDark ? 'bg-neutral-700' : 'bg-neutral-400/60'} rounded-lg px-4 py-3 space-y-2 mb-8 mt-8`}>
          <div className="flex items-center justify-between">
            <span className={isDark ? 'text-neutral-300' : 'text-neutral-900'}>当前硬件：</span>
            <span className={isDark ? 'text-neutral-300' : 'text-neutral-900'}>{currentDevice}</span>
          </div>
          <div className="flex items-center justify-between">
            <span className={isDark ? 'text-neutral-300' : 'text-neutral-900'}>当前序列号：</span>
            <span className={isDark ? 'text-neutral-300' : 'text-neutral-900'}>{serialNumber}</span>
          </div>
        </div>

        <div
          className={`${isDark ? 'bg-neutral-700' : 'bg-neutral-400/60'} rounded-lg px-4 py-3 mb-4 cursor-pointer`}
          onClick={toggleTheme}
        >
          <div className="flex items-center justify-between">
            <span className={isDark ? 'text-neutral-300' : 'text-neutral-900'}>主题：</span>
            <div className="flex items-center gap-2">
              <span className={isDark ? 'text-neutral-300' : 'text-neutral-900'}>{theme}</span>
              <ChevronDown className={`w-4 h-4 ${isDark ? 'text-neutral-300' : 'text-neutral-900'}`} />
            </div>
          </div>
        </div>
        <div className={`text-center ${isDark ? 'text-neutral-400' : 'text-neutral-900'}`}>
          {currentTime}
        </div>
      </div>
    </aside>
  );
}