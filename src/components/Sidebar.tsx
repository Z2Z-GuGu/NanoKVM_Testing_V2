import { useState, useEffect } from 'react';
import { ChevronDown } from 'lucide-react';

interface SidebarProps {
  machineCode: string;
  serverStatus: 'online' | 'offline';
  uploadCount: number;
  currentDevice: string;
  serialNumber: string;
  targetIP: string;
  theme: '明亮' | '暗黑';
  onThemeChange: (theme: '明亮' | '暗黑') => void;
  isDark: boolean;
}

export function Sidebar({
  machineCode,
  serverStatus,
  uploadCount,
  currentDevice,
  serialNumber,
  targetIP,
  theme,
  onThemeChange,
  isDark
}: SidebarProps) {
  const [currentTime, setCurrentTime] = useState('');

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

  const toggleTheme = () => {
    onThemeChange(theme === '明亮' ? '暗黑' : '明亮');
  };

  return (
    <aside className={`w-[280px] ${isDark ? 'bg-neutral-800' : 'bg-neutral-300'} p-5 flex flex-col justify-between`}>
      <div>
        <div className={'mb-16 mt-1'}>
          <div className="flex flex-col">
            <h1 className={`${isDark ? 'text-white' : 'text-neutral-900'}`} style={{ fontSize: '34px', fontWeight: 'bold', marginLeft: '4px', marginBottom: '16px' }}>NanoKVM-Pro</h1>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'baseline' }}>
              <h1 className={`${isDark ? 'text-white' : 'text-neutral-900'}`} style={{ fontSize: '34px', fontWeight: 'bold', lineHeight: '1.2', margin: 0, marginLeft: '4px' }}>产测工具</h1>
              <p className={'text-sm ' + (isDark ? 'text-neutral-400' : 'text-neutral-700')} style={{ fontSize: '20px', fontWeight: 'bold', margin: 0, verticalAlign: 'baseline', marginRight: '6px' }}>V2.0</p>
            </div>
          </div>
        </div>
        <div className={`${isDark ? 'bg-neutral-700' : 'bg-neutral-400/60'} rounded-lg px-4 py-3`}>
          <div className="flex items-center justify-between">
            <span className={isDark ? 'text-neutral-300' : 'text-neutral-900'}>本机编码：</span>
            <span className={isDark ? 'text-neutral-300' : 'text-neutral-900'}>{machineCode}</span>
          </div>
        </div>
      </div>

      <div>
        <div className={`${isDark ? 'bg-neutral-700' : 'bg-neutral-400/60'} rounded-lg px-4 py-3 space-y-2 mb-8 mt-8`}>
          <div className="flex items-center justify-between">
            <span className={isDark ? 'text-neutral-300' : 'text-neutral-900'}>服务器状态：</span>
            <div className="flex items-center gap-2">
              <div className={`w-3 h-3 rounded-full ${serverStatus === 'online' ? (isDark ? 'bg-green-700':'bg-green-300') : 'bg-neutral-500'}`} />
            </div>
          </div>
          <div className="flex items-center justify-between">
            <span className={isDark ? 'text-neutral-300' : 'text-neutral-900'}>待上传数量：</span>
            <span className={isDark ? 'text-neutral-300' : 'text-neutral-900'}>{uploadCount}</span>
          </div>
          <div className="flex items-center justify-between">
            <span className={isDark ? 'text-neutral-300' : 'text-neutral-900'}>当前硬件：</span>
            <span className={isDark ? 'text-neutral-300' : 'text-neutral-900'}>{currentDevice}</span>
          </div>
          <div className="flex items-center justify-between">
            <span className={isDark ? 'text-neutral-300' : 'text-neutral-900'}>当前序列号：</span>
            <span className={isDark ? 'text-neutral-300' : 'text-neutral-900'}>{serialNumber}</span>
          </div>
          <div className="flex items-center justify-between">
            <span className={isDark ? 'text-neutral-300' : 'text-neutral-900'}>目标IP：</span>
            <span className={isDark ? 'text-neutral-300' : 'text-neutral-900'}>{targetIP}</span>
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