#!/bin/bash

SAVE_DIR="/root/factory_test/done"
GPIO_PATH="/sys/class/gpio"
WS2812_IO="0"

init_io() {
    devmem 0x0230000C 32 0x00060003
    echo $WS2812_IO > $GPIO_PATH/export
    echo in > $GPIO_PATH/gpio$WS2812_IO/direction
}

io_test() {
    local overtime=$1
    local gpio_path="$GPIO_PATH/gpio$WS2812_IO/value"
    
    # 检测标志
    local high_detected=0
    local low_detected=0
    
    # 记录开始时间
    local start_time=$(date +%s)
    local current_time=$start_time
    local elapsed_time=0
    
    echo "开始检测WS2812_IO引脚电平变化..."
    
    while [ $elapsed_time -lt $overtime ]; do
        # 读取当前GPIO电平值
        if [ -f "$gpio_path" ]; then
            local current_value=$(cat "$gpio_path")
            
            # 检测高电平
            if [ "$current_value" = "1" ] && [ $high_detected -eq 0 ]; then
                echo "检测到高电平"
                high_detected=1
            fi
            
            # 检测低电平
            if [ "$current_value" = "0" ] && [ $low_detected -eq 0 ]; then
                echo "检测到低电平"
                low_detected=1
            fi
            
            # 如果已经检测到两种电平，立即返回成功
            if [ $high_detected -eq 1 ] && [ $low_detected -eq 1 ]; then
                echo "成功检测到高电平和低电平变化"
                return 0
            fi
        fi
        
        # 等待100毫秒后继续检测
        sleep 0.1
        
        # 更新已用时间
        current_time=$(date +%s)
        elapsed_time=$((current_time - start_time))
    done
    
    # 超时检查结果
    if [ $high_detected -eq 1 ] && [ $low_detected -eq 1 ]; then
        echo "成功检测到高电平和低电平变化"
        return 0
    else
        echo "超时：未检测到完整的高低电平变化"
        echo "高电平检测: $high_detected, 低电平检测: $low_detected"
        return 1
    fi
}

if [ ! -f "$SAVE_DIR/.io.done" ]; then
    init_io
    if io_test $1; then
        touch "$SAVE_DIR/.io.done"
        echo "IO test passed"
    else
        echo "IO test failed"
    fi
else
    echo "IO test passed"
fi

echo "Finish"

