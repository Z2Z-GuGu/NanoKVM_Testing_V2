import { SecondSidebar } from '../components/SencendSidebar';
import { SecondTestPanel } from '../components/SecondTestPanel';
import { FloatingButton } from '../components/FloatingButton';
import { DialogModal } from '../components/DialogModal';

interface SecondTestPageProps {
  theme: '明亮' | '暗黑';
  onThemeChange: (theme: '明亮' | '暗黑') => void;
  isDark: boolean;
  isTerminalOpen: boolean;
  onTerminalOpenChange: (isOpen: boolean) => void;
}

export function SecondTestPage({
  theme,
  onThemeChange,
  isDark,
  isTerminalOpen: _isTerminalOpen, // 未使用的属性，加下划线表示
  onTerminalOpenChange
}: SecondTestPageProps) {
  return (
    <>
      {/* 侧边栏 */}
      <SecondSidebar theme={theme} onThemeChange={onThemeChange} isDark={isDark} />
      {/* 后测面板 */}
      <SecondTestPanel isDark={isDark} />
      {/* 悬浮按钮 */}
      <FloatingButton onClick={() => onTerminalOpenChange(true)} isDark={isDark} />
      {/* 终端组件 */}
      {/* <Terminal isOpen={isTerminalOpen} onClose={() => onTerminalOpenChange(false)} isDark={isDark} /> */}
      {/* 弹窗组件 */}
      <DialogModal isDark={isDark} />
    </>
  );
}