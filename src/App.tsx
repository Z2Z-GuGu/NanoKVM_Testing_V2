import { useState } from 'react';
import { Sidebar } from './components/Sidebar';
import { TestPanel } from './components/TestPanel';

export default function App() {
  const [machineCode, setMachineCode] = useState('1');
  const [serverStatus, setServerStatus] = useState<'online' | 'offline'>('offline');
  const [uploadCount, setUploadCount] = useState(32);
  const [currentDevice, setCurrentDevice] = useState('Desk-A');
  const [serialNumber, setSerialNumber] = useState('Neal0015B');
  const [targetIP, setTargetIP] = useState('192.168.222.222');
  const [theme, setTheme] = useState<'明亮' | '暗黑'>('暗黑');

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
    </div>
  );
}