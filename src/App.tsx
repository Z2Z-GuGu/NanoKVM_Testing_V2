import { useState } from 'react';
import { SelectorPage } from './pages/SelectorPage';
import { TestPage } from './pages/TestPage';
import { SecondTestPage } from './pages/SecondTestPage';
import { selectProgram } from './services/backendService';

export default function App() {
  const [theme, setTheme] = useState<'明亮' | '暗黑'>('明亮');
  const [isTerminalOpen, setIsTerminalOpen] = useState(false);
  const [programSelected, setProgramSelected] = useState(false);
  const [selectedProgram, setSelectedProgram] = useState<'production' | 'post'>('production');

  const isDark = theme === '暗黑';

  // 处理程序选择
  const handleProgramSelect = (program: 'production' | 'post') => {
    setSelectedProgram(program);
    setProgramSelected(true);
    
    // 通知后端选择结果
    const notifyBackend = async () => {
      try {
        await selectProgram(program);
      } catch (error) {
        console.error('通知后端程序选择失败:', error);
      }
    };
    
    notifyBackend();
  };

  return (
    <div className={`flex h-screen ${isDark ? 'bg-neutral-900' : 'bg-neutral-100'}`}>
      {!programSelected ? (
        <SelectorPage onSelect={handleProgramSelect} isDark={isDark} />
      ) : (
        selectedProgram === 'production' ? (
          <TestPage 
            theme={theme} 
            onThemeChange={setTheme} 
            isDark={isDark}
            isTerminalOpen={isTerminalOpen}
            onTerminalOpenChange={setIsTerminalOpen}
          />
        ) : (
          <SecondTestPage 
            theme={theme} 
            onThemeChange={setTheme} 
            isDark={isDark}
            isTerminalOpen={isTerminalOpen}
            onTerminalOpenChange={setIsTerminalOpen}
          />
        )
      )}
    </div>
  );
}