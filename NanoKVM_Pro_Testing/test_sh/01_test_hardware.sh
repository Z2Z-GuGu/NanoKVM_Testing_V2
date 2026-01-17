#!/bin/bash

# 生成文件夹（防止自启）
mkdir -p /root/factory_test/done
mkdir -p /root/log

# 生成测试记录文件夹
mkdir /etc/test-kvm

# 加载6911ko
insmod /root/NanoKVM_Pro_Testing/ko/lt6911_manage.ko

# 加载屏幕驱动
insmod /root/NanoKVM_Pro_Testing/ko/fbtft.ko
insmod /root/NanoKVM_Pro_Testing/ko/fb_jd9853.ko

# 加载旋钮驱动
insmod /root/NanoKVM_Pro_Testing/ko/gpio_keys.ko
insmod /root/NanoKVM_Pro_Testing/ko/rotary_encoder.ko

cp /root/NanoKVM_Pro_Testing/config /root/

soc_id=$(cat /device_key)
echo "SOC ID: $soc_id"

# 判断是否存在旧产测中的串号
if [ -f /etc/test-kvm/serial ]; then
    # 存储到变量
    TEST_SERIAL=$(cat /etc/test-kvm/serial)
    # 如果为空还是NULL，就设为NULL
    if [ -z "$TEST_SERIAL" ]; then
        TEST_SERIAL="NULL"
    fi
else
    TEST_SERIAL="NULL"
fi

if BOARD_SERIAL=$(cat /proc/lt6911_info/version 2>/dev/null | awk -F') ' '{print $2}') && [ -n "$BOARD_SERIAL" ]; then
    : # 保持 BOARD_SERIAL 的值不变
else
    BOARD_SERIAL="Unknown"
fi

if [ "$BOARD_SERIAL" == "Unknown" ]; then
    if [ "$TEST_SERIAL" == "NULL" ]; then
        # 未产测过，从零开始
        echo "不弹窗，直接开始"
    else
        # 维修硬件，弹窗是否从零开始
        echo "弹窗内容：使用上次存储的串号，是否重新开始？"
        echo "当前板卡的串号为：$TEST_SERIAL"
    fi
    # 检测ATX/Desk版本
    echo "正在检测ATX/Desk版本..."
    # 执行i2cdetect命令并自动输入y
    I2C_RESULT=$(echo "y" | i2cdetect -ry 7 2>/dev/null)

    # 判断结果
    if echo "$I2C_RESULT" | grep -q "UU"; then
        echo "检测到Desk版本"
        BOARD_TYPE="Desk"
    elif echo "$I2C_RESULT" | grep -q "3c"; then
        echo "检测到ATX版本"
        BOARD_TYPE="ATX"
    else
        echo "未知版本"
        BOARD_TYPE="Unknown"
    fi

    echo "当前板卡的类型为：$BOARD_TYPE"
else
    if [ "$TEST_SERIAL" == "NULL" ]; then
        # 将当前板卡串号写入文件
        echo $BOARD_SERIAL > /etc/test-kvm/serial
        echo "弹窗内容：疑似更换核心板，使用当前底板串号，是否从零开始？"
        echo "当前板卡的串号为：$BOARD_SERIAL"
    else
        if [ "$BOARD_SERIAL" == "$TEST_SERIAL" ]; then
            echo "弹窗内容：再次测试，是否从零开始产测？"
            echo "当前板卡的串号为：$BOARD_SERIAL"
        else
            echo $BOARD_SERIAL > /etc/test-kvm/serial
            echo "弹窗内容：疑似更换核心板，使用当前底板串号，是否从零开始？"
            echo "当前板卡的串号为：$BOARD_SERIAL"
        fi
    fi
    BOARD_TYPE=$(cat /proc/lt6911_info/version | awk -F'[()]' '{print $2}')
    echo "当前板卡的类型为：$BOARD_TYPE"
fi

# 检测是否有wifi模块
if [ -f /etc/test-kvm/wifi_exist ]; then
    echo "当前板卡有wifi模块"
else
    # 通过ip a | grep wlan0判断是否存在wifi模块
    if ip a | grep -q "wlan0"; then
        echo "当前板卡有wifi模块"
        # 创建标记文件，避免重复检测
        touch /etc/test-kvm/wifi_exist
    else
        echo "当前板卡无wifi模块"
    fi
fi

# 获取所有hw_开头的文件（只获取文件名）
shopt -s nullglob
files=(/etc/test-kvm/hw_*)

if [ ${#files[@]} -eq 0 ]; then
    echo "[]"
else
    # 提取文件名并格式化输出
    output="["
    for file in "${files[@]}"; do
        if [ -f "$file" ]; then
            output+=" $(basename "$file")"
        fi
    done
    output+=" ]"
    echo "$output"
fi

sync

echo "Finish"




