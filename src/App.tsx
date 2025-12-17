import { useState } from 'react';
import { Sidebar } from './components/Sidebar';
import { TestPanel } from './components/TestPanel';
import { Terminal } from './components/Terminal';
import { FloatingButton } from './components/FloatingButton';
import { DialogModal, DialogButton} from "./components/DialogModal";

export default function App() {
  const [theme, setTheme] = useState<'明亮' | '暗黑'>('明亮');
  const [isTerminalOpen, setIsTerminalOpen] = useState(false);

  // 弹窗状态
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [dialogMessage, setDialogMessage] = useState("");
  const [dialogButtons, setDialogButtons] = useState< DialogButton[] >([]);

  const isDark = theme === '暗黑';

  // 测试弹窗功能（开发模式）
  const showTestDialog = () => {
    setDialogMessage(
      "这是一个提示，可能是多行的，可以由后端指定，按钮数量/内容也是由后端指定，不知道能不能自动换行，测试可以",
    );
    setDialogButtons([
      { text: "OK" },
      { text: "不OK" },
    ]);
    setIsDialogOpen(true);
  };

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
      {/* 测试弹窗按钮（开发模式） - 左下角 */}
      <button
        onClick={showTestDialog}
        className={`fixed bottom-8 left-8 px-4 py-2 rounded-lg shadow-lg transition-all duration-200 z-30 ${
          isDark
            ? "bg-blue-600 hover:bg-blue-500 text-white"
            : "bg-blue-500 hover:bg-blue-600 text-white"
        }`}
      > 测试弹窗 </button>
      {/* 弹窗组件 */}
      <DialogModal isOpen={isDialogOpen} onClose={() => setIsDialogOpen(false)} isDark={isDark} message={dialogMessage} buttons={dialogButtons}/>
    </div>
  );
}