#!/bin/bash

# ./07_eth_test.sh download 500 "http://192.168.1.7:8080/download"
# ./07_eth_test.sh upload 350 http://192.168.1.7:8080/upload

SAVE_DIR="/root/factory_test/done"

eth_download_test() {
    net_mini_speed=$1
    url=$2
    echo "开始以太网下载测试, 速度阈值: $net_mini_speed"
    
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

eth_upload_test() {
    net_mini_speed=$1
    url=$2
    echo "开始以太网上传测试, 速度阈值: $net_mini_speed"
    
    # 将Mb/s转换为B/s (1 Mb/s = 125000 B/s)
    min_speed_bps=$(echo "$net_mini_speed * 125000" | bc)
    
    # 创建测试文件
    if [ ! -f /tmp/test.bin ]; then
        echo "创建测试文件..."
        dd if=/dev/zero of=/tmp/test.bin bs=1M count=10 2>/dev/null
    fi
    
    # 执行上传测试并获取速度
    speed_result=$(curl -X POST "$url" --data-binary @/tmp/test.bin -s -o /dev/null -w "%{speed_upload}")
    
    # 检查curl命令是否成功执行
    if [ $? -ne 0 ]; then
        echo "上传测试失败"
        rm -f /tmp/test.bin
        return 1
    fi
    
    # 输出实际速度
    echo "实际上传速度: $speed_result B/s"
    
    # 比较速度（使用bc进行浮点数比较）
    if [ $(echo "$speed_result > $min_speed_bps" | bc) -eq 1 ]; then
        echo "上传速度测试通过"
        # 清理临时文件
        rm -f /tmp/test.bin
        return 0
    else
        echo "上传速度测试失败，要求速度: $min_speed_bps B/s，实际速度: $speed_result B/s"
        # 清理临时文件
        rm -f /tmp/test.bin
        return 1
    fi
}

case "$1" in
    download)
        if [ ! -f "$SAVE_DIR/.eth_download.done" ]; then
            if eth_download_test $2 $3; then
                touch "$SAVE_DIR/.eth_download.done"
                echo "ETH download test passed"
            else
                echo "ETH download test failed"
            fi
        else
            echo "ETH download test passed"
        fi
        ;;
    upload)
        if [ ! -f "$SAVE_DIR/.eth_upload.done" ]; then
            if eth_upload_test $2 $3; then
                touch "$SAVE_DIR/.eth_upload.done"
                echo "ETH upload test passed"
            else
                echo "ETH upload test failed"
            fi
        else
            echo "ETH upload test passed"
        fi
        ;;
    *)
        echo "Usage: $0 <download|upload> spped url"
        ;;
esac

sync

echo "Finish"




