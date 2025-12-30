#!/bin/bash

SAVE_DIR="/root/factory_test/done"
GPIO_PATH="/sys/class/gpio"
PWR_KEY_IO="7"
RST_KEY_IO="35"
PWR_LED_IO="75"
HDD_LED_IO="74"

init_io() {
    echo $PWR_KEY_IO > $GPIO_PATH/export
    echo $RST_KEY_IO > $GPIO_PATH/export
    echo $PWR_LED_IO > $GPIO_PATH/export
    echo $HDD_LED_IO > $GPIO_PATH/export

    echo out > $GPIO_PATH/gpio$PWR_KEY_IO/direction
    echo out > $GPIO_PATH/gpio$RST_KEY_IO/direction
    echo in > $GPIO_PATH/gpio$PWR_LED_IO/direction
    echo in > $GPIO_PATH/gpio$HDD_LED_IO/direction
}

desk_io_test() {
    # 0//0 = 0
    echo 0 > $GPIO_PATH/gpio$PWR_KEY_IO/value
    echo 0 > $GPIO_PATH/gpio$RST_KEY_IO/value
    sleep 0.2
    if [ ! "$(cat $GPIO_PATH/gpio$PWR_LED_IO/value)" = "0" ]; then
        return 1
    fi
    # 0//1 = 0
    echo 0 > $GPIO_PATH/gpio$PWR_KEY_IO/value
    echo 1 > $GPIO_PATH/gpio$RST_KEY_IO/value
    sleep 0.2
    if [ ! "$(cat $GPIO_PATH/gpio$PWR_LED_IO/value)" = "0" ]; then
        return 1
    fi
    # 1//0 = 0
    echo 1 > $GPIO_PATH/gpio$PWR_KEY_IO/value
    echo 0 > $GPIO_PATH/gpio$RST_KEY_IO/value
    sleep 0.2
    if [ ! "$(cat $GPIO_PATH/gpio$PWR_LED_IO/value)" = "0" ]; then
        return 1
    fi
    # 1//1 = 1
    echo 1 > $GPIO_PATH/gpio$PWR_KEY_IO/value
    echo 1 > $GPIO_PATH/gpio$RST_KEY_IO/value
    sleep 0.2
    if [ ! "$(cat $GPIO_PATH/gpio$PWR_LED_IO/value)" = "1" ]; then
        return 1
    fi

    return 0
}

atx_io_test() {
    # PWR KEY = 0:PWR LED = 1
    echo 0 > $GPIO_PATH/gpio$PWR_KEY_IO/value
    sleep 0.2
    if [ ! "$(cat $GPIO_PATH/gpio$PWR_LED_IO/value)" = "1" ]; then
        echo "ATX引脚报错：PWR KEY/PWR LED IO 错误"
        return 1
    fi
    # PWR KEY = 1:PWR LED = 0
    echo 1 > $GPIO_PATH/gpio$PWR_KEY_IO/value
    sleep 0.2
    if [ ! "$(cat $GPIO_PATH/gpio$PWR_LED_IO/value)" = "0" ]; then
        echo "ATX引脚报错：PWR KEY/PWR LED IO 错误"
        return 1
    fi
    # RST KEY = 0:HDD LED = 1
    echo 0 > $GPIO_PATH/gpio$RST_KEY_IO/value
    sleep 0.2
    if [ ! "$(cat $GPIO_PATH/gpio$HDD_LED_IO/value)" = "1" ]; then
        echo "ATX引脚报错：RST KEY/HDD LED IO 错误"
        return 1
    fi
    # RST KEY = 1:HDD LED = 0
    echo 1 > $GPIO_PATH/gpio$RST_KEY_IO/value
    sleep 0.2
    if [ ! "$(cat $GPIO_PATH/gpio$HDD_LED_IO/value)" = "0" ]; then
        echo "ATX引脚报错：RST KEY/HDD LED IO 错误"
        return 1
    fi

    return 0
}

case "$1" in
    desk)
        if [ ! -f "$SAVE_DIR/.atx.done" ]; then
            init_io
            if desk_io_test; then
                touch "$SAVE_DIR/.atx.done"
                echo "ATX test passed"
            else
                echo "ATX test failed"
            fi
        else
            echo "ATX test passed"
        fi
        ;;
    atx)
        if [ ! -f "$SAVE_DIR/.atx.done" ]; then
            init_io
            if atx_io_test; then
                touch "$SAVE_DIR/.atx.done"
                echo "ATX test passed"
            else
                echo "ATX test failed"
            fi
        else
            echo "ATX test passed"
        fi
        ;;
    *)
        echo "Usage: $0 <desk|atx>"
        ;;
esac

sync

echo "Finish"

