#!/bin/bash

FIRMWARE_DIR="/root/NanoKVM_Pro_Testing/firmware"
KO_DIR="/root/NanoKVM_Pro_Testing/ko"
SAVE_DIR="/root/factory_test/done"

init_vin() {
    if [ ! -f /tmp/vin.pid ]; then
        $FIRMWARE_DIR/kvm_vin_test &
        echo $! > /tmp/vin.pid
    fi
}

deinit_vin() {
    if [ -f /tmp/vin.pid ]; then
        kill $(cat /tmp/vin.pid)
        rm /tmp/vin.pid
    fi
}

# 测试所有IO状态的函数
test_all_io() {
    echo "=== 开始测试所有IO状态 ==="
    local all_tests_passed=true

    # 判断/proc/lt6911_info目录是否存在
    if [ ! -d /proc/lt6911_info ]; then
        echo "/proc/lt6911_info目录不存在"
        all_tests_passed=false
        return 1 # 直接返回测试失败
    fi
    
    # 测试HDMI电源状态
    local hdmi_power=$(cat /proc/lt6911_info/hdmi_power)
    if [ "$hdmi_power" = "on" ]; then
        echo "LT86102 RST 引脚正常"
    else
        echo "LT86102 RST 引脚异常"
        all_tests_passed=false
    fi
    
    # 测试电源状态
    local power=$(cat /proc/lt6911_info/power)
    if [ "$power" = "on" ]; then
        echo "LT6911 RST 引脚正常"
    else
        echo "LT6911 RST 引脚异常"
        all_tests_passed=false
    fi
    
    # 测试HDMI RX状态
    local hdmi_rx_status=$(cat /proc/lt6911_info/hdmi_rx_status)
    if [ "$hdmi_rx_status" = "access" ]; then
        echo "LT86102 RX 引脚正常"
    else
        echo "LT86102 RX 引脚异常"
        all_tests_passed=false
    fi
    
    # 测试HDMI TX状态
    local hdmi_tx_status=$(cat /proc/lt6911_info/hdmi_tx_status)
    if [ "$hdmi_tx_status" = "access" ]; then
        echo "LT86102 TX 引脚正常"
    else
        echo "LT86102 TX 引脚异常"
        all_tests_passed=false
    fi
    
    # 测试状态
    local status=$(cat /proc/lt6911_info/status)
    if [ "$status" = "new res" ]; then
        echo "LT6911 INT 引脚正常"
    else
        echo "LT6911 INT 引脚异常"
        all_tests_passed=false
    fi
    
    # 测试I2C引脚
    I2C_RESULT=$(echo "y" | i2cdetect -ry 0 2>/dev/null)
    # 判断结果
    if echo "$I2C_RESULT" | grep -q "UU"; then
        echo "LT6911 I2C 引脚正常"
    else
        echo "LT6911 I2C 引脚异常"
        all_tests_passed=false
    fi
    if echo "$I2C_RESULT" | grep -q "38"; then
        echo "LT86102 I2C 引脚正常"
    else
        echo "LT86102 I2C 引脚异常"
        all_tests_passed=false
    fi

    # 输出最终测试结果
    if [ "$all_tests_passed" = true ]; then
        return 0
    else
        return 1
    fi
}

# 测试VIN状态的函数
test_vin() {
    out_fps=$(awk '/^\[CHN\]/ {chn=1; next} chn && NF && $1 ~ /^[0-9]+$/ {print $6; exit}' /proc/ax_proc/vin/statistics)
    # 检查out_fps是否为空值
    if [ -z "$out_fps" ]; then
        echo "FPS值为空，VIN测试失败"
        return 1
    fi
    
    # 检查out_fps是否为有效数字（支持小数）
    if ! echo "$out_fps" | grep -qE '^[0-9]+(\.[0-9]+)?$'; then
        echo "FPS值格式错误: $out_fps，VIN测试失败"
        return 1
    fi
    
    # 使用bc进行浮点数比较，判断是否大于0
    if echo "$out_fps > 0" | bc -l | grep -q 1; then
        echo "FPS值正常: $out_fps，VIN测试通过"
        return 0
    else
        echo "FPS值为0或负数: $out_fps，VIN测试失败"
        return 1
    fi
}

# 写入Version
write_version() {
    local version=$1
    echo "写入版本号: $version"
    echo "$version" > /proc/lt6911_info/version
    sleep 1
    local version2=$(cat /proc/lt6911_info/version)
    
    if [ "$version" = "$version2" ]; then
        return 0
    else
        return 1
    fi
}

# 写入EDID
write_edid() {
    cat $FIRMWARE_DIR/edid_e56.bin > /proc/lt6911_info/edid

    sleep 1

    local edid1=$(xxd /proc/lt6911_info/edid)
    local edid2=$(xxd $FIRMWARE_DIR/edid_e56.bin)
    
    if [ "$edid1" = "$edid2" ]; then
        return 0
    else
        echo $edid1
        echo "-----------------"
        echo $edid2
        return 1
    fi
}

# 维修LT6911
repair_lt6911() {
    rmmod lt6911_manage
    echo "repair lt6911"
    $FIRMWARE_DIR/nanokvm_update_6911 $FIRMWARE_DIR/nanokvm_6911.bin
    insmod $KO_DIR/lt6911_manage.ko
}

# 维修LT86102
repair_lt86102() {
    rmmod lt6911_manage
    echo "repair lt86102"
    $FIRMWARE_DIR/nanokvm_update_86102 $FIRMWARE_DIR/nanokvm_86102R1[68].bin
    insmod $KO_DIR/lt6911_manage.ko
}

# 执行对应函数
case "$1" in
    start)
        if [ ! -f "$SAVE_DIR/.hdmi_vin.done" ]; then
            init_vin
        fi
        ;;
    io)
        if [ ! -f "$SAVE_DIR/.hdmi_io.done" ]; then
            if test_all_io; then
                touch "$SAVE_DIR/.hdmi_io.done"
                echo "HDMI IO test passed"
            else
                echo "HDMI IO test failed"
            fi
        else
            echo "HDMI IO test passed"
        fi
        ;;
    vin)
        if [ ! -f "$SAVE_DIR/.hdmi_vin.done" ]; then
            if test_vin; then
                touch "$SAVE_DIR/.hdmi_vin.done"
                deinit_vin
                echo "HDMI VIN test passed"
            else
                echo "HDMI VIN test failed"
            fi
        else
            deinit_vin
            echo "HDMI VIN test passed"
        fi
        ;;
    version)
        if [ ! -f "$SAVE_DIR/.hdmi_version.done" ]; then
            if write_version "$2"; then
                touch "$SAVE_DIR/.hdmi_version.done"
                echo "HDMI version write passed"
            else
                echo "HDMI version write failed"
            fi
        else
            echo "HDMI version write passed"
        fi
        ;;
    edid)
        if [ ! -f "$SAVE_DIR/.hdmi_edid.done" ]; then
            if write_edid; then
                touch "$SAVE_DIR/.hdmi_edid.done"
                echo "HDMI EDID write passed"
            else
                echo "HDMI EDID write failed"
            fi
        else
            echo "HDMI EDID write passed"
        fi
        ;;
    repair_lt6911)
        repair_lt6911
        echo "HDMI LT6911 repaired"
        ;;
    repair_lt86102)
        repair_lt86102
        echo "HDMI LT86102 repaired"
        ;;
    *)
        echo "Usage: $0 <start|io|vin|version|edid|repair_lt6911|repair_lt86102>"
        ;;
esac

sync

echo "Finish"
exit 0


