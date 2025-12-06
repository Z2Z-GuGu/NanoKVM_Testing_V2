import { useEffect, useRef, useState } from 'react';

interface TerminalProps {
  isOpen: boolean;
  onClose: () => void;
  isDark: boolean;
}

export function Terminal({ isOpen, onClose, isDark }: TerminalProps) {
  const [output, setOutput] = useState<string[]>([]);
  const [isClosing, setIsClosing] = useState(false);
  const [isAnimating, setIsAnimating] = useState(false);
  const terminalRef = useRef<HTMLDivElement>(null);

  // 处理打开动画
  useEffect(() => {
    if (isOpen) {
      // 延迟一帧以触发CSS过渡
      requestAnimationFrame(() => {
        setIsAnimating(true);
      });
    }
  }, [isOpen]);

  // 处理关闭动画
  const handleClose = () => {
    setIsClosing(true);
    setIsAnimating(false);
    setTimeout(() => {
      setIsClosing(false);
      onClose();
    }, 300); // 与动画时长一致
  };

  // 自动滚动到底部
  useEffect(() => {
    if (terminalRef.current) {
      terminalRef.current.scrollTop = terminalRef.current.scrollHeight;
    }
  }, [output]);

  // 监听来自Tauri后端的消息
  useEffect(() => {
    if (!isOpen) return;

    // 这里使用Tauri的事件监听
    // @ts-ignore - Tauri API
    if (window.__TAURI__) {
      // @ts-ignore
      const { listen } = window.__TAURI__.event;
      
      const unlisten = listen('terminal-output', (event: any) => {
        setOutput(prev => [...prev, event.payload]);
      });

      return () => {
        unlisten.then((fn: any) => fn());
      };
    } else {
      // 开发模式下的模拟数据
      const mockData = [
        '[ 13.160778] fbift_of_value: fps = 90',
        '[ 13.160878] fb_jd9853 spi2.1: fbift_request_one_gpio: \'reset-gpios\' = GPI041',
        '[ 13.160895] fb_jd9853 spi2.1: fbift_request_one_gpio: \'de-gpios\' = GPI043',
        '[ 13.160927] fb_jd9853 spi2.1: debug mode: off',
        '[ 13.756609] using random self ethernet address',
        '[ 13.756617] using random host ethernet address',
        '[ 13.160778] fbift_of_value: fps = 90',
        '[ 13.160878] fb_jd9853 spi2.1: fbift_request_one_gpio: \'reset-gpios\' = GPI041',
        '[ 13.160895] fb_jd9853 spi2.1: fbift_request_one_gpio: \'de-gpios\' = GPI043',
        '[ 13.160927] fb_jd9853 spi2.1: debug mode: off',
        '[ 13.756609] using random self ethernet address',
        '[ 13.756617] using random host ethernet address',
        '[ 13.160778] fbift_of_value: fps = 90',
        '[ 13.160878] fb_jd9853 spi2.1: fbift_request_one_gpio: \'reset-gpios\' = GPI041',
        '[ 13.160895] fb_jd9853 spi2.1: fbift_request_one_gpio: \'de-gpios\' = GPI043',
        '[ 13.160927] fb_jd9853 spi2.1: debug mode: off',
        '[ 13.756609] using random self ethernet address',
        '[ 13.756617] using random host ethernet address',
        '[ 13.160778] fbift_of_value: fps = 90',
      ];
      
      setOutput(mockData);
    }
  }, [isOpen]);

  // 监听全局键盘事件
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyPress = (e: KeyboardEvent) => {
      // ESC键关闭终端
      if (e.key === 'Escape') {
        handleClose();
        return;
      }

      // 捕获所有键盘输入发送到后端
      // @ts-ignore
      if (window.__TAURI__) {
        // @ts-ignore
        const { invoke } = window.__TAURI__.tauri;
        invoke('send_terminal_key', { key: e.key });
      }
    };

    window.addEventListener('keydown', handleKeyPress);
    return () => window.removeEventListener('keydown', handleKeyPress);
  }, [isOpen]);

  if (!isOpen && !isClosing) return null;

  return (
    <>
      {/* 背景遮罩 */}
      <div 
        className={`fixed inset-0 transition-opacity duration-300 z-40 bg-black/60 ${
          isClosing ? 'opacity-0' : 'opacity-100'
        }`}
        onClick={handleClose}
      />

      {/* 终端窗口 - 从底部弹起，左右留10%，上面留10%，下面贴底 */}
      <div 
        className={`fixed z-50 transition-transform duration-300 shadow-2xl overflow-hidden rounded-t-lg ${
          isAnimating ? 'translate-y-0' : 'translate-y-full'
        }`}
        style={{
          left: '10%',
          right: '10%',
          top: '10%',
          bottom: 0,
          backgroundColor: isDark ? '#262626' : '#E5E5E5',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* 终端输出区域 - 全屏可输入 */}
        <div 
          ref={terminalRef}
          className="w-full h-full overflow-y-auto px-6 py-4"
          style={{
            marginTop: '8px',
            height: '97%',
          }}
        >
          <div className="font-mono text-sm space-y-1">
            {output.map((line, index) => (
              <div 
                key={index}
                className={isDark ? 'text-neutral-300' : 'text-neutral-800'}
              >
                {line}
              </div>
            ))}
          </div>
        </div>
      </div>
    </>
  );
}