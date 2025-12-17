import { useEffect, useState } from 'react';

export interface DialogButton {
  text: string;
  isPrimary?: boolean; // 是否为主按钮（深色按钮）
  onClick?: () => void;
}

interface DialogModalProps {
  isOpen: boolean;
  onClose: () => void;
  isDark: boolean;
  message: string;
  buttons: DialogButton[];
}

export function DialogModal({ isOpen, onClose, isDark, message, buttons }: DialogModalProps) {
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
      onClose();
    }, 200);
  };

  // 处理按钮点击
  const handleButtonClick = (button: DialogButton) => {
    if (button.onClick) {
      button.onClick();
    }
    handleClose();
  };

  // 监听ESC键关闭弹窗
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyPress = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        handleClose();
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
        className={`fixed inset-0 transition-opacity duration-200 z-40 bg-black/60 ${
          isClosing ? 'opacity-0' : 'opacity-100'
        }`}
        onClick={handleClose}
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
