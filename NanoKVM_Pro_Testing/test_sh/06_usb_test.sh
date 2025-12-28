#!/bin/bash


SAVE_DIR="/root/factory_test/done"

usb_test() {
    echo "开始USB测试..."
    
    # 读取USB状态
    local usb_state=$(cat /sys/class/udc/8000000.dwc3/state)
    echo "当前USB状态: $usb_state"
    
    # 判断状态是否为configured
    if [ "$usb_state" = "configured" ]; then
        echo "✓ USB测试通过 - 设备已正确配置"
        return 0
    else
        echo "✗ USB测试失败 - 期望状态: configured, 实际状态: $usb_state"
        return 1
    fi
}

if [ ! -f "$SAVE_DIR/.usb.done" ]; then
    if usb_test; then
        echo "USB test passed"
        touch "$SAVE_DIR/.usb.done"
    else
        echo "USB test failed"
    fi
else
    echo "USB test passed"
fi

echo "Finish"


