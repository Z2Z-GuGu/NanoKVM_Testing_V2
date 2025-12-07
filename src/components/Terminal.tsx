import { useEffect, useRef, useState } from 'react';
import { Terminal as XTerminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';

interface TerminalProps {
  isOpen: boolean;
  onClose: () => void;
  isDark: boolean;
}

export function Terminal({ isOpen, onClose, isDark }: TerminalProps) {
  const [isClosing, setIsClosing] = useState(false);
  const [isAnimating, setIsAnimating] = useState(false);
  const terminalRef = useRef<HTMLDivElement>(null);
  const terminalInstanceRef = useRef<XTerminal | null>(null);
  const fitAddonRef = useRef<FitAddon | null>(null);
  const wsRef = useRef<WebSocket | null>(null);

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

  // 初始化终端
  useEffect(() => {
    if (!isOpen || !terminalRef.current) return;

    // 初始化xterm.js终端
    const terminal = new XTerminal({
      fontSize: 12,
      fontFamily: 'monospace',
      theme: {
        foreground: isDark ? '#FFFFFF' : '#000000',
        background: isDark ? '#262626' : '#E5E5E5',
        cursor: isDark ? '#FFFFFF' : '#000000',
      },
      lineHeight: 1.2,
    });

    // 初始化fit addon
    const fitAddon = new FitAddon();
    terminal.loadAddon(fitAddon);

    // 打开终端
    terminal.open(terminalRef.current);

    // 调整终端大小以适应容器
    setTimeout(() => {
      fitAddon.fit();
    }, 0);

    // 保存终端实例和fit addon
    terminalInstanceRef.current = terminal;
    fitAddonRef.current = fitAddon;

    // 处理窗口大小变化
    const handleResize = () => {
      if (fitAddonRef.current && terminalInstanceRef.current) {
        fitAddonRef.current.fit();
      }
    };

    window.addEventListener('resize', handleResize);

    // 处理终端输入
    terminal.onData((data) => {
      // 发送数据到后端
      if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
        wsRef.current.send(data);
      }
    });

    return () => {
      window.removeEventListener('resize', handleResize);
      terminal.dispose();
    };
  }, [isOpen, isDark]);

  // 建立WebSocket连接
  useEffect(() => {
    if (!isOpen) return;

    // 建立WebSocket连接
    const ws = new WebSocket('ws://localhost:8080');
    
    ws.onopen = () => {
      console.log('WebSocket连接已建立');
    };

    ws.onmessage = (event) => {
      // 接收来自后端的数据并显示在终端
      if (terminalInstanceRef.current) {
        terminalInstanceRef.current.write(event.data);
      }
    };

    ws.onclose = () => {
      console.log('WebSocket连接已关闭');
      // 显示断开连接消息
      if (terminalInstanceRef.current) {
        terminalInstanceRef.current.write('[disconnected]\n');
      }
    };

    ws.onerror = (error) => {
      console.error('WebSocket错误:', error);
    };

    // 保存WebSocket实例
    wsRef.current = ws;

    return () => {
      ws.close();
    };
  }, [isOpen]);

  // 监听Tauri事件（用于开发模式）
  useEffect(() => {
    if (!isOpen) return;

    // 这里使用Tauri的事件监听（开发模式下的模拟数据）
    // @ts-ignore - Tauri API
    if (window.__TAURI__) {
      // @ts-ignore
      const { listen } = window.__TAURI__.event;
      
      const unlisten = listen('terminal-output', (event: any) => {
        if (terminalInstanceRef.current) {
          terminalInstanceRef.current.write(event.payload);
        }
      });

      return () => {
        unlisten.then((fn: any) => fn());
      };
    } else {
      // 开发模式下的模拟数据
      const mockData = [
        '[connected]\n',
        '[ 13.160778] fbift_of_value: fps = 90\n',
        '[ 13.160878] fb_jd9853 spi2.1: fbift_request_one_gpio: \'reset-gpios\' = GPI041\n',
        '[ 13.160895] fb_jd9853 spi2.1: fbift_request_one_gpio: \'de-gpios\' = GPI043\n',
        '[ 13.160927] fb_jd9853 spi2.1: debug mode: off\n',
      ];
      
      if (terminalInstanceRef.current) {
        mockData.forEach(line => {
          terminalInstanceRef.current?.write(line);
        });
      }
    }
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
          className="w-full h-full"
          style={{
            marginTop: '8px',
            height: '97%',
          }}
        />
      </div>
    </>
  );
}