#!/bin/bash

SAVE_DIR="/root/factory_test/done"

uart_test() {
    local uart_num="$1"
    local uart_dev="/dev/ttyS${uart_num}"
    local test_str="NANOKVM_LOOPBACK_TEST"

    if [ ! -e "$uart_dev" ]; then
        echo "$uart_dev not found. Please check connection"
        return 1
    fi

    echo "Start UART loopback $uart_dev (TX ↔ RX)..."

    stty -F "$uart_dev" 115200 cs8 -cstopb -parenb -echo raw

    local tmp_recv="/tmp/uart_recv_$uart_num.txt"
    rm -f "$tmp_recv"

    timeout 2 cat "$uart_dev" > "$tmp_recv" &
    local cat_pid=$!

    sleep 0.1

    echo -n "$test_str" > "$uart_dev"
    wait "$cat_pid" || true

    local recv_str
    recv_str=$(cat "$tmp_recv")

    if [ "$recv_str" = "$test_str" ]; then
        return 0
    else
        echo "串口$uart_num-测试失败"
        return 1
    fi
}

if [ ! -f "$SAVE_DIR/.uart.done" ]; then
    uart_test 1
    uart1_result=$?
    uart_test 2  
    uart2_result=$?
    if [ $uart1_result -eq 0 ] && [ $uart2_result -eq 0 ]; then
        touch "$SAVE_DIR/.uart.done"
        echo "UART test passed"
    else
        echo "UART test failed"
    fi
else
    echo "UART test passed"
fi

sync

echo "Finish"

