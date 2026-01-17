#!/bin/bash


SAVE_DIR="/root/factory_test/done"

usb_test() {
    echo "开始USB测试..."
    
    # 启动后台进程，每0.1秒向串口0发送UUUUUUUUUU
    (while true; do 
        echo -n "UUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUUU" > /dev/ttyS0
        sleep 0.1
    done) &
    
    # 保存后台进程PID
    local uart_sender_pid=$!
    
    # 初始化计数器
    local consecutive_passes=0
    local max_checks=10  # 最多检查10次
    
    # 使用for循环明确控制检查次数
    for ((i=1; i<=max_checks; i++)); do
        # 读取USB状态
        local usb_state=$(cat /sys/class/udc/8000000.dwc3/state)
        echo "第 $i/$max_checks 次检查 - 当前USB状态: $usb_state"
        
        # 判断状态是否为configured
        if [ "$usb_state" = "configured" ]; then
            consecutive_passes=$((consecutive_passes + 1))
            echo "连续通过次数: $consecutive_passes"
        else
            consecutive_passes=0
            echo "USB状态异常，连续通过次数重置为0"
        fi
        
        # 如果连续通过次数达到5次，提前结束测试
        if [ $consecutive_passes -ge 5 ]; then
            echo "已连续检测到5次configured状态，提前结束测试"
            break
        fi
        
        # 除了最后一次检查外，每次检查后等待1秒
        if [ $i -lt $max_checks ]; then
            sleep 1
        fi
    done
    
    # 停止后台进程
    kill $uart_sender_pid 2>/dev/null
    wait $uart_sender_pid 2>/dev/null
    
    # 判断测试结果
    if [ $consecutive_passes -ge 5 ]; then
        echo "✓ USB测试通过 - 连续5次检测到configured状态"
        return 0
    else
        echo "✗ USB测试失败 - 在$max_checks次检查中未连续检测到5次configured状态"
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

sync

echo "Finish"


