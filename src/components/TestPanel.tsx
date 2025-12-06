import { TestButton } from './TestButton';

interface TestPanelProps {
  isDark: boolean;
}

export function TestPanel({ isDark }: TestPanelProps) {
  return (
    <main className="flex-1 p-8 overflow-auto">
      {/* 初始化 */}
      <section className="mb-8">
        <h2 className={`mb-4 ${isDark ? 'text-white' : 'text-neutral-900'}`} style={{ fontSize: '24px', fontWeight: 'bold' }}>初始化</h2>
        <div className="flex flex-wrap gap-3">
          <TestButton status="untested" isDark={isDark}>等待连接</TestButton>
          <TestButton status="testing" isDark={isDark}>等待开机</TestButton>
          <TestButton status="repairing" isDark={isDark}>获取IP</TestButton>
          <TestButton status="success" isDark={isDark}>检测硬件</TestButton>
          <TestButton status="failed" isDark={isDark}>下载产测</TestButton>
        </div>
      </section>

      <hr className={`mb-8 ${isDark ? 'border-neutral-700' : 'border-neutral-300'}`} />

      {/* 项目测试 */}
      <section className="mb-8">
        <h2 className={`mb-4 ${isDark ? 'text-white' : 'text-neutral-900'}`} style={{ fontSize: '24px', fontWeight: 'bold' }}>项目测试</h2>
        
        {/* 系统更新 */}
        <div className="flex flex-wrap gap-3 mb-3">
          <div className="w-24 flex items-center">
            <span className={isDark ? 'text-white' : 'text-neutral-900'}>系统更新</span>
          </div>
          <TestButton status="untested" isDark={isDark}>uboot</TestButton>
          <TestButton status="untested" isDark={isDark}>kernel</TestButton>
          <TestButton status="untested" isDark={isDark}>APP安装</TestButton>
        </div>

        {/* HDMI接口 */}
        <div className="flex flex-wrap gap-3 mb-3">
          <div className="w-24 flex items-center">
            <span className={isDark ? 'text-white' : 'text-neutral-900'}>HDMI接口</span>
          </div>
          <TestButton status="untested" isDark={isDark}>等待连接</TestButton>
          <TestButton status="untested" isDark={isDark}>测试环出</TestButton>
          <TestButton status="untested" isDark={isDark}>测试采集</TestButton>
          <TestButton status="repairing" isDark={isDark}>写EDID</TestButton>
          <TestButton status="untested" isDark={isDark}>写EDID</TestButton>
          <TestButton status="untested" isDark={isDark}>Version</TestButton>
        </div>

        {/* USB接口 */}
        <div className="flex flex-wrap gap-3 mb-3">
          <div className="w-24 flex items-center">
            <span className={isDark ? 'text-white' : 'text-neutral-900'}>USB接口</span>
          </div>
          <TestButton status="untested" isDark={isDark}>等待连接</TestButton>
        </div>

        {/* ETH接口 */}
        <div className="flex flex-wrap gap-3 mb-3">
          <div className="w-24 flex items-center">
            <span className={isDark ? 'text-white' : 'text-neutral-900'}>ETH接口</span>
          </div>
          <TestButton status="untested" isDark={isDark}>等待连接</TestButton>
          <TestButton status="untested" isDark={isDark}>上传测速</TestButton>
          <TestButton status="untested" isDark={isDark}>下载测速</TestButton>
        </div>

        {/* WiFi */}
        <div className="flex flex-wrap gap-3 mb-3">
          <div className="w-24 flex items-center">
            <span className={isDark ? 'text-white' : 'text-neutral-900'}>WiFi</span>
          </div>
          <TestButton status="untested" isDark={isDark}>等待连接</TestButton>
          <TestButton status="untested" isDark={isDark}>上传测速</TestButton>
          <TestButton status="untested" isDark={isDark}>下载测速</TestButton>
        </div>

        {/* 交互 */}
        <div className="flex flex-wrap gap-3 mb-3">
          <div className="w-24 flex items-center">
            <span className={isDark ? 'text-white' : 'text-neutral-900'}>交互</span>
          </div>
          <TestButton status="untested" isDark={isDark}>屏幕</TestButton>
          <TestButton status="untested" isDark={isDark}>触摸</TestButton>
          <TestButton status="untested" isDark={isDark}>旋钮</TestButton>
        </div>

        {/* 其他 */}
        <div className="flex flex-wrap gap-3 mb-3">
          <div className="w-24 flex items-center">
            <span className={isDark ? 'text-white' : 'text-neutral-900'}>其他</span>
          </div>
          <TestButton status="untested" isDark={isDark}>ATX</TestButton>
          <TestButton status="untested" isDark={isDark}>IO</TestButton>
          <TestButton status="untested" isDark={isDark}>TF卡</TestButton>
        </div>
      </section>

      <hr className={`mb-8 ${isDark ? 'border-neutral-700' : 'border-neutral-300'}`} />

      {/* 应用设置 */}
      <section>
        <h2 className={`mb-4 ${isDark ? 'text-white' : 'text-neutral-900'}`} style={{ fontSize: '24px', fontWeight: 'bold' }}>应用设置</h2>
        <div className="flex gap-3">
          <TestButton status="untested" isDark={isDark}>开机自启</TestButton>
        </div>
      </section>
    </main>
  );
}