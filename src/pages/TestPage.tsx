import { Sidebar } from '../components/Sidebar';
import { TestPanel } from '../components/TestPanel';
import { Terminal } from '../components/Terminal';
import { FloatingButton } from '../components/FloatingButton';
import { DialogModal } from '../components/DialogModal';

interface TestPageProps {
  theme: '明亮' | '暗黑';
  onThemeChange: (theme: '明亮' | '暗黑') => void;
  isDark: boolean;
  isTerminalOpen: boolean;
  onTerminalOpenChange: (isOpen: boolean) => void;
}

export function TestPage({
  theme,
  onThemeChange,
  isDark,
  isTerminalOpen,
  onTerminalOpenChange
}: TestPageProps) {
  return (
    <>
      {/* 侧边栏 */}
      <Sidebar theme={theme} onThemeChange={onThemeChange} isDark={isDark} />
      {/* 测试面板 */}
      <TestPanel isDark={isDark} />
      {/* 悬浮按钮 */}
      <FloatingButton onClick={() => onTerminalOpenChange(true)} isDark={isDark} />
      {/* 终端组件 */}
      <Terminal isOpen={isTerminalOpen} onClose={() => onTerminalOpenChange(false)} isDark={isDark} />
      {/* 弹窗组件 */}
      <DialogModal isDark={isDark} />
    </>
  );
}