#!/bin/bash

log_info()  { echo -e "\e[32m[INFO] $*\e[0m"; }
log_warn()  { echo -e "\e[33m[WARN] $*\e[0m"; }
log_error() { echo -e "\e[31m[ERROR] $*\e[0m"; }
log_step()  { echo -e "\e[1;34m===> $*\e[0m"; }
log_debug() {
    if [ "$DEBUG" == "True" ]; then
        echo -e "\e[1;36m[DEBUG] $*\e[0m"
    fi
}

emmc_test() {
    log_info "Starting eMMC badblocks test"
    if [ -f /root/factory_test/done/.emmc.done ]; then
        log_info "eMMC test already completed, skipping."
        return 0
    fi
    set +e

    local emmc_device="/dev/mmcblk0"

    # 1GB = 1024*1024*1024 bytes = 1073741824 bytes
    # 4KB = 4096 bytes
    # 1GB / 4KB = 262144 blocks
    local test_blocks=262144

    log_debug "Running: badblocks -sv -b 4096 $emmc_device $test_blocks"

    local badblocks_output="/root/log/emmc_badblocks.log"
    local badblocks_result="/root/log/emmc_badblocks_result.txt"
    local dmesg_log="/root/log/emmc_dmesg.log"

    dmesg -c > /dev/null 2>&1

    timeout 300 badblocks -sv -b 4096 -o "$badblocks_result" "$emmc_device" "$test_blocks" > "$badblocks_output" 2>&1
    local badblocks_exit_code=$?

    dmesg > "$dmesg_log"
    if grep -q "print_req_error: I/O error" "$dmesg_log"; then
        log_error "Kernel reported I/O errors during eMMC test:"
        grep "print_req_error: I/O error" "$dmesg_log" | while read -r line; do
            log_error "  $line"
        done
        rm -f "$badblocks_output" "$badblocks_result" "$dmesg_log"
        return 1
    fi

    if [ $badblocks_exit_code -eq 124 ]; then
        log_warn "badblocks test timed out after 5 minutes"
        rm -f "$badblocks_output" "$badblocks_result" "$dmesg_log"
        return 1
    elif [ $badblocks_exit_code -ne 0 ]; then
        log_error "badblocks test failed with exit code $badblocks_exit_code"
        if [ -f "$badblocks_output" ]; then
            log_error "badblocks error output:"
            tail -5 "$badblocks_output" | while read line; do
                log_error "  $line"
            done
        fi
        rm -f "$badblocks_output" "$badblocks_result" "$dmesg_log"
        return 1
    fi

    local bad_blocks_count=0
    if [ -f "$badblocks_result" ] && [ -s "$badblocks_result" ]; then
        bad_blocks_count=$(wc -l < "$badblocks_result")
        log_error "Found $bad_blocks_count bad blocks in tested 1GB area:"
        while read -r bad_block; do
            log_error "  Bad block: $bad_block"
        done < "$badblocks_result"
        rm -f "$badblocks_output" "$badblocks_result" "$dmesg_log"
        return 1
    fi

    if [ -f "$badblocks_output" ]; then
        local last_line
        last_line=$(tail -1 "$badblocks_output")
        if [ -n "$last_line" ]; then
            log_info "Test result: $last_line"
        fi
    fi

    rm -f "$badblocks_output" "$badblocks_result" "$dmesg_log"
    touch "/root/factory_test/done/.emmc.done"
    log_info "eMMC badblocks test completed successfully"
    set -e

    return 0
}

# 测试eMMC
EMMC=emmc_test
if $EMMC; then
    echo "eMMC test passed"
else
    echo "eMMC test failed"
fi

sync

echo "Finish"


