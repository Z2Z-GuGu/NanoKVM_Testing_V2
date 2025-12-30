#!/bin/bash

FIRMWARE_DIR="/root/NanoKVM_Pro_Testing/firmware"
SAVE_DIR="/root/factory_test/done"

cd /root

lcdtest() {
    local test_type=$1
    local overtime=$2

    # 根据测试类型选择不同的参数
    local test_command="$FIRMWARE_DIR/kvm_spi_ui_test"
    case "$test_type" in
        "lcd")
            # 使用默认命令，不加参数
            ;;
        "touch")
            test_command="$FIRMWARE_DIR/kvm_spi_ui_test --touch"
            ;;
        "rotary")
            test_command="$FIRMWARE_DIR/kvm_spi_ui_test --rotary"
            ;;
        "oled")
            test_command="$FIRMWARE_DIR/kvm_ui_test"
            ;;
        *)
            echo "未知的测试类型: $test_type"
            return 1
            ;;
    esac

    # 启动测试程序
    $test_command &
    local pid=$!
    echo $pid > /tmp/$test_type.pid
    
    # 等待程序运行，检测是否在超时时间内自动结束
    local wait_time=0
    while [ $wait_time -lt $overtime ]; do
        # 检查进程是否还在运行
        if ! ps -p $pid > /dev/null 2>&1; then
            # 程序已自动结束，返回成功
            return 0
        fi
        
        sleep 1
        wait_time=$((wait_time + 1))
    done
    
    if ps -p $pid > /dev/null 2>&1; then
        kill $pid 2>/dev/null
        return 1
    else
        return 0
    fi
}

case "$1" in
    lcd)
        lcdtest lcd $2
        ;;
    touch)
        if lcdtest touch $2; then
            echo "Touch test passed"
        else
            echo "Touch test failed"
        fi
        ;;
    rotary)
        if [ ! -f "$SAVE_DIR/.rotary.done" ]; then
            if lcdtest rotary $2; then
                touch "$SAVE_DIR/.rotary.done"
                echo "Rotary test passed"
            else
                echo "Rotary test failed"
            fi
        else
            echo "Rotary test passed"
        fi
        ;;
    oled)
        lcdtest oled $2
        ;;
    *)
        echo "Usage: $0 <lcd|oled|touch|rotary>"
        ;;
esac

sync

echo "Finish"




