import { useState } from 'react';
import { Sidebar } from './components/Sidebar';
import { TestPanel } from './components/TestPanel';
import { Terminal } from './components/Terminal';
import { Plus } from 'lucide-react';

export default function App() {
  const [machineCode, setMachineCode] = useState('1');
  const [serverStatus, setServerStatus] = useState<'online' | 'offline'>('offline');
  const [uploadCount, setUploadCount] = useState(32);
  const [currentDevice, setCurrentDevice] = useState('Desk-A');
  const [serialNumber, setSerialNumber] = useState('Neal0015B');
  const [targetIP, setTargetIP] = useState('192.168.222.222');
  const [theme, setTheme] = useState<'明亮' | '暗黑'>('暗黑');
  const [isTerminalOpen, setIsTerminalOpen] = useState(false);

  const isDark = theme === '暗黑';

  return (
    <div className={`flex h-screen ${isDark ? 'bg-neutral-900' : 'bg-neutral-100'}`}>
      <Sidebar
        machineCode={machineCode}
        serverStatus={serverStatus}
        uploadCount={uploadCount}
        currentDevice={currentDevice}
        serialNumber={serialNumber}
        targetIP={targetIP}
        theme={theme}
        onThemeChange={setTheme}
        isDark={isDark}
      />
      <TestPanel isDark={isDark} />
      {/* 右下角圆形"+"按钮 */}
      <button
        onClick={() => setIsTerminalOpen(true)}
        className={`fixed bottom-8 right-8 w-14 h-14 rounded-full shadow-lg transition-all duration-200 flex items-center justify-center z-30 ${
          isDark
            ? 'bg-neutral-700 hover:bg-neutral-600 text-neutral-200'
            : 'bg-neutral-800 hover:bg-neutral-700 text-white'
        }`}
      >
        <Plus className="w-6 h-6" />
      </button>

      {/* 终端组件 */}
      <Terminal 
        isOpen={isTerminalOpen} 
        onClose={() => setIsTerminalOpen(false)}
        isDark={isDark}
      />
    </div>
  );
}