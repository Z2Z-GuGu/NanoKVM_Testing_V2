#!/bin/bash

# ./07_eth_test.sh download 500 "http://192.168.1.7:8080/download"
# ./07_eth_test.sh upload 350 http://192.168.1.7:8080/upload

SAVE_DIR="/root/factory_test/done"

wifi_connect() {
    ssid=$1
    pass=$2
    if [ ! -e /tmp/wpa_supplicant.conf ]; then
        # wpa_passphrase "NanoKVM_WiFi_Test_1" "nanokvmwifi" >>/tmp/wpa_supplicant.conf
        wpa_passphrase "$ssid" "$pass" >>/tmp/wpa_supplicant.conf
    fi
    wpa_supplicant -B -i wlan0 -c /tmp/wpa_supplicant.conf
    if [ ! -e /tmp/udhcpc.wlan0.pid ]; then
        udhcpc_output=$(udhcpc -i wlan0 -t 2 -b -p /tmp/udhcpc.wlan0.pid 2>&1)
        
        # 检查是否连接失败（输出包含"background"）
        if echo "$udhcpc_output" | grep -q "background"; then
            echo "DHCP连接失败: $udhcpc_output"
            return 1
        fi
        
        local_ip=$(echo "$udhcpc_output" | grep -oP "(?<=select for )\d+\.\d+\.\d+\.\d+")
        server_ip=$(echo "$udhcpc_output" | grep -oP "(?<=obtained from )\d+\.\d+\.\d+\.\d+")

        echo "本机IP: $local_ip"
        echo "DHCP服务器IP: $server_ip"

        return 0
    else
        local_ip=$(ip -4 -o addr show wlan0 2>/dev/null | awk '{print $4}' | cut -d'/' -f1)
        if [[ -n "$local_ip" ]] && [[ "$local_ip" =~ ^[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}$ ]]; then
            # 将本机IP最后一位改为1作为服务器IP
            server_ip=$(echo "$local_ip" | sed 's/\.[0-9]*$/\.1/')
            echo "本机IP: $local_ip"
            echo "DHCP服务器IP: $server_ip"
            return 0
        else
            return 1
        fi
    fi
}

wifi_download_test() {
    net_mini_speed=$1
    url=$2
    echo "开始WiFi下载测试, 速度阈值: $net_mini_speed"
    
    # 将Mb/s转换为B/s (1 Mb/s = 125000 B/s)
    min_speed_bps=$(echo "$net_mini_speed * 125000" | bc)
    
    # 执行下载测试并获取速度
    speed_result=$(curl "$url" --output /dev/null -s -o /dev/null -w "%{speed_download}")
    
    # 检查curl命令是否成功执行
    if [ $? -ne 0 ]; then
        echo "下载测试失败"
        return 1
    fi
    
    # 输出实际速度
    echo "实际下载速度: $speed_result B/s"
    
    # 比较速度（使用bc进行浮点数比较）
    if [ $(echo "$speed_result > $min_speed_bps" | bc) -eq 1 ]; then
        echo "下载速度测试通过"
        return 0
    else
        echo "下载速度测试失败，要求速度: $min_speed_bps B/s，实际速度: $speed_result B/s"
        return 1
    fi
}

wifi_upload_test() {
    net_mini_speed=$1
    url=$2
    echo "开始WiFi上传测试, 速度阈值: $net_mini_speed"
    
    # 将Mb/s转换为B/s (1 Mb/s = 125000 B/s)
    min_speed_bps=$(echo "$net_mini_speed * 125000" | bc)
    
    # 创建测试文件
    if [ ! -f /tmp/test5.bin ]; then
        echo "创建测试文件..."
        dd if=/dev/zero of=/tmp/test5.bin bs=1M count=5 2>/dev/null
    fi
    
    # 执行上传测试并获取速度
    speed_result=$(curl -X POST "$url" --data-binary @/tmp/test5.bin -s -o /dev/null -w "%{speed_upload}")
    
    # 检查curl命令是否成功执行
    if [ $? -ne 0 ]; then
        echo "上传测试失败"
        rm -f /tmp/test5.bin
        return 1
    fi
    
    # 输出实际速度
    echo "实际上传速度: $speed_result B/s"
    
    # 比较速度（使用bc进行浮点数比较）
    if [ $(echo "$speed_result > $min_speed_bps" | bc) -eq 1 ]; then
        echo "上传速度测试通过"
        # 清理临时文件
        rm -f /tmp/test5.bin
        return 0
    else
        echo "上传速度测试失败，要求速度: $min_speed_bps B/s，实际速度: $speed_result B/s"
        # 清理临时文件
        rm -f /tmp/test5.bin
        return 1
    fi
}

case "$1" in
    connect)
        # ./08_wifi_test.sh connect "NanoKVM_WiFi_Test_1" "nanokvmwifi"
        if [ -f "$SAVE_DIR/.wifi_download.done" ] && [ -f "$SAVE_DIR/.wifi_upload.done" ]; then
            echo "WiFi connect passed"
        else
            if wifi_connect $2 $3; then
                echo "WiFi connect passed"
            else
                echo "WiFi connect failed"
            fi
        fi
        ;;
    download)
        if [ ! -f "$SAVE_DIR/.wifi_download.done" ]; then
            if wifi_download_test $2 $3; then
                touch "$SAVE_DIR/.wifi_download.done"
                echo "WiFi download test passed"
            else
                echo "WiFi download test failed"
            fi
        else
            echo "WiFi download test passed"
        fi
        ;;
    upload)
        if [ ! -f "$SAVE_DIR/.wifi_upload.done" ]; then
            if wifi_upload_test $2 $3; then
                touch "$SAVE_DIR/.wifi_upload.done"
                echo "WiFi upload test passed"
            else
                echo "WiFi upload test failed"
            fi
        else
            echo "WiFi upload test passed"
        fi
        ;;
    *)
        echo "Usage: $0 <download|upload> spped url"
        ;;
esac

echo "Finish"

