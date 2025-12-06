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
    const base = 'w-32 px-6 py-2 rounded-md transition-all duration-300 relative ';
    
    if (status === 'success') {
      return base + (isDark 
        ? 'bg-green-700 text-neutral-200 hover:bg-green-700' 
        : 'bg-green-200 text-neutral-900 hover:bg-green-300');
    }
    
    if (status === 'failed') {
      return base + (isDark 
        ? 'bg-red-700 text-neutral-200 hover:bg-red-700' 
        : 'bg-red-200 text-neutral-900 hover:bg-red-300');
    }
    
    // 未测试、正在测试、正在修复使用中性色
    return base + (isDark 
      ? 'bg-neutral-700 text-neutral-200 hover:bg-neutral-600' 
      : 'bg-neutral-200 text-neutral-900 hover:bg-neutral-300');
  };

  // 获取边框动画样式
  const getBorderAnimation = () => {
    if (status === 'testing') {
      const greenColor = isDark ? '#16a34a' : '#22c55e';
      return (
        <>
          <span 
            className="absolute inset-0 rounded-md overflow-hidden pointer-events-none"
            style={{
              padding: '2px',
            }}
          >
            <span 
              className="absolute inset-0 rounded-md"
              style={{
                top: '-120%',
                left: '-50%',
                right: '-50%',
                bottom: '-120%',
                background: `conic-gradient(from 0deg, transparent 0deg, transparent 0deg, ${greenColor} 270deg, ${greenColor} 360deg, transparent 360deg)`,
                animation: 'borderRotate 2.5s linear infinite',
              }}
            />
            <span 
              className={`absolute inset-[2px] rounded-md ${
                isDark ? 'bg-neutral-700' : 'bg-neutral-200'
              }`}
            />
          </span>
        </>
      );
    }
    
    if (status === 'repairing') {
      const yellowColor = isDark ? '#ca8a04' : '#eab308';
      return (
        <>
          <span 
            className="absolute inset-0 rounded-md overflow-hidden pointer-events-none"
            style={{
              padding: '2px',
            }}
          >
            <span 
              className="absolute inset-0 rounded-md"
              style={{
                top: '-120%',
                left: '-50%',
                right: '-50%',
                bottom: '-120%',
                background: `conic-gradient(from 0deg, transparent 0deg, transparent 0deg, ${yellowColor} 270deg, ${yellowColor} 360deg, transparent 360deg)`,
                animation: 'borderRotate 2.5s linear infinite',
              }}
            />
            <span 
              className={`absolute inset-[2px] rounded-md ${
                isDark ? 'bg-neutral-700' : 'bg-neutral-200'
              }`}
            />
          </span>
        </>
      );
    }
    
    return null;
  };

  return (
    <button
      onClick={onClick}
      className={`${getBaseClasses()} overflow-visible`}
      disabled={status === 'testing' || status === 'repairing'}
    >
      {/* 动画边框 */}
      {getBorderAnimation()}
      
      {/* 主要内容 */}
      <span className="relative z-10">{children}</span>
      
      {/* 添加 CSS 动画 */}
      <style>{`
        @keyframes borderRotate {
          from {
            transform: rotate(0deg);
          }
          to {
            transform: rotate(360deg);
          }
        }
      `}</style>
    </button>
  );
}