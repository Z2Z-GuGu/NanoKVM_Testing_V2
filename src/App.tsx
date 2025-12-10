import { useState } from 'react';
import { Sidebar } from './components/Sidebar';
import { TestPanel } from './components/TestPanel';
import { Terminal } from './components/Terminal';
import { FloatingButton } from './components/FloatingButton';

export default function App() {
  const [theme, setTheme] = useState<'明亮' | '暗黑'>('明亮');
  const [isTerminalOpen, setIsTerminalOpen] = useState(false);

  const isDark = theme === '暗黑';

  return (
    <div className={`flex h-screen ${isDark ? 'bg-neutral-900' : 'bg-neutral-100'}`}>
      {/* 侧边栏 */}
      <Sidebar theme={theme} onThemeChange={setTheme} isDark={isDark}/>
      {/* 测试面板 */}
      <TestPanel isDark={isDark}/>
      {/* 悬浮按钮 */}
      <FloatingButton onClick={() => setIsTerminalOpen(true)} isDark={isDark}/>
      {/* 终端组件 */}
      <Terminal isOpen={isTerminalOpen} onClose={() => setIsTerminalOpen(false)} isDark={isDark}/>
    </div>
  );
}