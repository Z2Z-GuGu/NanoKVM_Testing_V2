import { useEffect, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

// 扩展 Window 类型以包含 __TAURI__
declare global {
  interface Window {
    __TAURI__?: any;
  }
}

export interface DialogButton {
  text: string;
  isPrimary?: boolean; // 是否为主按钮（深色按钮）
  onClick?: () => void;
}

interface DialogModalProps {
  isDark: boolean;
}

// 开发环境检测 - 移到组件内部以便动态检测

export function DialogModal({ isDark }: DialogModalProps) {
  // 内部管理弹窗状态
  const [isOpen, setIsOpen] = useState(false);
  const [message, setMessage] = useState("");
  const [buttons, setButtons] = useState<DialogButton[]>([]);
  const [isClosing, setIsClosing] = useState(false);
  const [isAnimating, setIsAnimating] = useState(false);

  // 处理打开动画
  useEffect(() => {
    if (isOpen) {
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
      setIsOpen(false);
      // 清空状态
      setMessage("");
      setButtons([]);
    }, 200);
  };

  // 处理按钮点击
  const handleButtonClick = (button: DialogButton) => {
    console.log('DialogModal: 按钮被点击:', button.text);
    
    // 调用后端命令，将按钮文本发送到后端
    invoke('handle_button_click', {
      buttonText: button.text
    }).then(() => {
      console.log('DialogModal: 后端已接收到按钮点击事件:', button.text);
    }).catch(error => {
      console.error('DialogModal: 调用后端命令失败:', error);
    });
    
    if (button.onClick) {
      button.onClick();
    }
    handleClose();
  };

  // 监听后端弹窗事件 - 使用与Sidebar相同的简单实现方式
  useEffect(() => {
    console.log('DialogModal: 开始监听后端弹窗事件...');
    
    // 直接监听弹窗事件，与Sidebar组件保持一致的实现方式
    const unlistenShow = listen('show-dialog', (event) => {
      console.log('DialogModal: 接收到弹窗事件:', event);
      console.log('DialogModal: 事件负载:', event.payload);
      
      try {
        // 解析事件数据
        const { message, buttons } = event.payload as {
          message: string;
          buttons: DialogButton[];
        };
        
        console.log('DialogModal: 解析后的弹窗数据:', { message, buttons });
        
        // 更新状态显示弹窗
        setMessage(message || '');
        setButtons(buttons || [{ text: '确定' }]);
        setIsOpen(true);
        
        console.log('DialogModal: 弹窗已显示');
      } catch (parseError) {
        console.error('DialogModal: 解析弹窗事件数据失败:', parseError);
      }
    }).catch(error => {
      console.error('DialogModal: 设置弹窗事件监听器失败:', error);
    });
    
    
    // 监听关闭弹窗事件
    const unlistenHide = listen('hide-dialog', (event) => {
      console.log('DialogModal: 接收到关闭弹窗事件:', event);
      console.log('DialogModal: 事件负载:', event.payload);
      
      try {
        // 关闭弹窗
        handleClose();
        console.log('DialogModal: 弹窗已关闭');
      } catch (parseError) {
        console.error('DialogModal: 处理关闭弹窗事件失败:', parseError);
      }
    }).catch(error => {
      console.error('DialogModal: 设置关闭弹窗事件监听器失败:', error);
    });

    // 清理函数
    return () => {
      console.log('DialogModal: 清理事件监听器');
      // 清理监听器
      if (unlistenShow) unlistenShow.then(unlisten => unlisten());
      if (unlistenHide) unlistenHide.then(unlisten => unlisten());
    };
  }, []);

  if (!isOpen && !isClosing) return null;

  return (
    <>
      {/* 背景遮罩 */}
      <div 
        className={`fixed inset-0 transition-opacity duration-200 z-40 bg-black/60 ${
          isClosing ? 'opacity-0' : 'opacity-100'
        }`}
      />

      {/* 弹窗内容 */}
      <div 
        className={`fixed z-50 transition-all duration-200 rounded-lg shadow-2xl p-8 ${
          isAnimating && !isClosing ? 'opacity-100 scale-100' : 'opacity-0 scale-95'
        }`}
        style={{
          left: '50%',
          top: '50%',
          transform: 'translate(-50%, -50%)',
          maxWidth: '90%',
          width: '700px',
          backgroundColor: isDark ? '#404040' : '#D9D9D9',
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* 提示文本 */}
        <div 
          className={`mb-8 whitespace-pre-wrap ${
            isDark ? 'text-white' : 'text-neutral-800'
          }`}
        >
          {message}
        </div>

        {/* 按钮组 */}
        <div className="flex gap-4 justify-end">
          {buttons.map((button, index) => (
            <button
              key={index}
              onClick={() => handleButtonClick(button)}
              className={`px-8 py-3 rounded-lg transition-all duration-200 ${
                  isDark
                  ? 'bg-neutral-800 hover:bg-neutral-600 text-white'
                  : 'bg-neutral-700 hover:bg-neutral-500 text-white'
              }`}
            >
              {button.text}
            </button>
          ))}
        </div>
      </div>
    </>
  );
}
