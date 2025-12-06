import { ReactNode } from 'react';

interface TestButtonProps {
  status: 'untested' | 'testing' | 'repairing' | 'success' | 'failed';
  children: ReactNode;
  onClick?: () => void;
  isDark: boolean;
}

export function TestButton({ status = 'untested', children, onClick, isDark }: TestButtonProps) {
  // 获取基础背景和文本颜色
  const getBaseClasses = () => {
    const base = 'w-32 px-6 py-2 rounded-md transition-all duration-300 font-medium ';
    
    if (status === 'success') {
      return base + (isDark 
        ? 'bg-green-700 text-neutral-200 hover:bg-green-700' 
        : 'bg-green-200 text-neutral-900 hover:bg-green-700');
    }
    
    if (status === 'failed') {
      return base + (isDark 
        ? 'bg-red-700 text-neutral-200 hover:bg-red-700' 
        : 'bg-red-200 text-neutral-900 hover:bg-red-700');
    }
    
    // 未测试、正在测试、正在修复使用中性色
    return base + (isDark 
      ? 'bg-neutral-700 text-neutral-200 hover:bg-neutral-600' 
      : 'bg-neutral-200 text-neutral-900 hover:bg-neutral-300');
  };

  return (
    <button
      onClick={onClick}
      className={`relative ${getBaseClasses()} flex flex-col items-center justify-center`}
      disabled={status === 'testing' || status === 'repairing'}
    >
      
      {/* 主要内容 */}
      <span className="relative z-10">{children}</span>
    </button>
  );
}