import testImg1 from '../assets/test_img_1.jpg';
import testImg2 from '../assets/test_img_2.jpg';

interface SelectorPageProps {
  onSelect: (program: 'production' | 'post') => void;
  isDark: boolean;
}

export function SelectorPage({ onSelect, isDark }: SelectorPageProps) {
  return (
    <div className="flex-1 flex flex-col items-center justify-center p-5 min-h-screen">
      {/* 大标题 - 正上方居中加粗 */}
      <h1 
        className={`mb-8 ${isDark ? 'text-white' : 'text-neutral-900'}`} 
        style={{ 
          fontSize: '36px', 
          fontWeight: 'bold', 
          textAlign: 'center',
          letterSpacing: '1px'
        }}
      >
        请选择测试形式 :
      </h1>
      
      {/* 按键容器 - 左右布局 */}
      <div className="flex flex-row gap-16 w-full max-w-5xl justify-center">
        {/* 产测程序按键 */}
        <button
          onClick={() => onSelect('production')}
          className={`flex flex-col items-center rounded-3xl transition-all duration-300 transform ${isDark 
            ? 'bg-transparent hover:bg-gray-800 text-white' 
            : 'bg-transparent hover:bg-gray-200 text-neutral-900'}`}
          style={{ 
            padding: '32px', 
            width: '350px',
            border: 'none',
            outline: 'none',
            transform: 'translateY(0)',
            transition: 'all 0.3s ease',
            cursor: 'pointer'
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.transform = 'translateY(-5px)';
            const imgContainer = e.currentTarget.querySelector('.img-container') as HTMLElement;
            if (imgContainer) {
              imgContainer.style.boxShadow = '0 10px 15px -3px rgba(255, 175, 70, 0.5), 0 4px 6px -4px rgba(59, 130, 246, 0.3)';
            }
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.transform = 'translateY(0)';
            const imgContainer = e.currentTarget.querySelector('.img-container') as HTMLElement;
            if (imgContainer) {
              imgContainer.style.boxShadow = '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1)';
            }
          }}
        >
          {/* 图片部分 - 圆角矩形蒙版 */}
          <div 
            className="img-container rounded-2xl overflow-hidden w-full mb-8 shadow-lg"
            style={{ 
              backgroundColor: isDark ? '#1f2937' : '#f3f4f6',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              overflow: 'hidden',
              width: '300px',
              height: '300px',
              boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1)',
              transition: 'box-shadow 0.3s ease'
            }}
          >
            <img 
              src={testImg1} 
              alt="KVM产测程序" 
              className="transition-transform duration-500 hover:scale-105"
              style={{ 
                width: '100%',
                height: '100%',
                objectFit: 'contain'
              }}
            />
          </div>
          
          {/* 文字部分 */}
          <span 
            className="text-2xl font-semibold"
            style={{ 
              textAlign: 'center',
              letterSpacing: '0.5px',
              fontSize: '22px',
              fontWeight: 'bold'
            }}
          >
            KVM产测程序
          </span>
        </button>
        
        {/* 后测程序按键 */}
        <button
          onClick={() => onSelect('post')}
          className={`flex flex-col items-center rounded-3xl transition-all duration-300 transform ${isDark 
            ? 'bg-transparent hover:bg-gray-800 text-white' 
            : 'bg-transparent hover:bg-gray-200 text-neutral-900'}`}
          style={{ 
            padding: '32px', 
            width: '350px',
            border: 'none',
            outline: 'none',
            transform: 'translateY(0)',
            transition: 'all 0.3s ease',
            cursor: 'pointer'
          }}
          onMouseEnter={(e) => {
            e.currentTarget.style.transform = 'translateY(-5px)';
            const imgContainer = e.currentTarget.querySelector('.img-container') as HTMLElement;
            if (imgContainer) {
              imgContainer.style.boxShadow = '0 10px 15px -3px rgba(22, 211, 245, 0.36), 0 4px 6px -4px rgba(4, 75, 116, 0.3)';
            }
          }}
          onMouseLeave={(e) => {
            e.currentTarget.style.transform = 'translateY(0)';
            const imgContainer = e.currentTarget.querySelector('.img-container') as HTMLElement;
            if (imgContainer) {
              imgContainer.style.boxShadow = '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1)';
            }
          }}
        >
          {/* 图片部分 - 圆角矩形蒙版 */}
          <div 
            className="img-container rounded-2xl overflow-hidden w-full mb-8 shadow-lg"
            style={{ 
              backgroundColor: isDark ? '#1f2937' : '#f3f4f6',
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              overflow: 'hidden',
              width: '300px',
              height: '300px',
              boxShadow: '0 4px 6px -1px rgba(8, 239, 255, 0.34), 0 2px 4px -2px rgba(0, 0, 0, 0.1)',
              transition: 'box-shadow 0.3s ease'
            }}
          >
            <img 
              src={testImg2} 
              alt="KVM后测程序" 
              className="transition-transform duration-500 hover:scale-105"
              style={{ 
                width: '100%',
                height: '100%',
                objectFit: 'contain'
              }}
            />
          </div>
          
          {/* 文字部分 */}
          <span 
            className="text-2xl font-semibold"
            style={{ 
              textAlign: 'center',
              letterSpacing: '0.5px',
              fontSize: '22px',
              fontWeight: 'bold'
            }}
          >
            KVM后测程序
          </span>
        </button>
      </div>
    </div>
  );
}