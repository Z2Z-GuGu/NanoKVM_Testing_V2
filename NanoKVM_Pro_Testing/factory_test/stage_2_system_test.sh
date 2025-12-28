#!/bin/bash

LOGFILE="/root/log/stage_2_system_test.log"
exec > >(tee -a "$LOGFILE") 2>&1

set -e

log_info()  { echo -e "\e[32m[INFO] $*\e[0m"; }
log_warn()  { echo -e "\e[33m[WARN] $*\e[0m"; }
log_error() { echo -e "\e[31m[ERROR] $*\e[0m"; }
log_step()  { echo -e "\e[1;34m===> $*\e[0m"; }
log_debug() {
    if [ "$DEBUG" == "True" ]; then
        echo -e "\e[1;36m[DEBUG] $*\e[0m"
    fi
}

send_msg() {
    local msg=$1
    local max_retries=5
    local attempt=1

    while [ $attempt -le $max_retries ]; do
        if echo "$msg" | timeout 2 nc "$HOST_IP" 2333 2>/dev/null; then
            return 0
        fi

        log_warn "Failed to send status (attempt $attempt/$max_retries): $msg"
        attempt=$((attempt + 1))
        sleep 1
    done

    log_error "Giving up after $max_retries attempts: $msg"
    return 1
}

get_msg() {
    local port="${1:-2334}"
    local timeout_sec="${2:-10}"
    local listen_ip="${3:-0.0.0.0}"

    log_debug "Listening for message on ${listen_ip}:${port} (timeout: ${timeout_sec}s)"

    local received_msg
    received_msg=$(timeout "$timeout_sec" nc -l -p "$port" 2>/dev/null | head -n 1)
    local exit_code=$?

    if [ $exit_code -eq 0 ] && [ -n "$received_msg" ]; then
        log_debug "Successfully received message: '$received_msg'"
        echo "$received_msg"
        return 0
    elif [ $exit_code -eq 124 ]; then
        log_warn "Timeout waiting for message on port $port after ${timeout_sec}s"
        return 1
    else
        log_warn "Failed to receive message on port $port (exit code: $exit_code)"
        return 2
    fi
}

CATEGORY="emmc"

fail_handler() {
    log_error "$CATEGORY update failed!"
    send_msg "${CATEGORY},fail"

    pkill -f kvm_ui_test || true
    pkill -f kvm_spi_ui_test || true
    pkill -f kvm_vin_test || true

    if check_wifi_interface; then
        check_wifi_mode_stop
    fi

    exit 1
}

trap fail_handler ERR

report_completed_tests() {
    local done_dir="/root/factory_test/done"
    local all_tests=("eth" "uart" "atx" "mipi" "screen" "touch" "rotary" "wifi" "sd" "emmc" "app")
    local completed_tests=()

    log_debug "Collecting completed test information..."
    if [ ! -d "$done_dir" ]; then
        log_debug "Done directory not found, no completed tests to report"
        send_msg "dev,completed:"
        return 0
    fi

    for test in "${all_tests[@]}"; do
        local done_file="$done_dir/.${test}.done"
        if [ -f "$done_file" ]; then
            completed_tests+=("$test")
        fi
    done

    local completed_list=""
    if [ ${#completed_tests[@]} -gt 0 ]; then
        completed_list=$(IFS=','; echo "${completed_tests[*]}")
        log_debug "Reporting completed tests to host: ${completed_tests[*]}"
    else
        log_debug "No completed tests to report"
    fi

    send_msg "dev,completed:$completed_list"

    return 0
}

firmware_hdmi() {
    log_info "Start HDMI firmware update"

    if lsmod | grep -q '^lt6911_manage'; then
        log_debug "Unloading lt6911_manage module"
        rmmod lt6911_manage 2>/dev/null || {
            log_error "Failed to unload lt6911_manage"
            return 1
        }
    fi

    local update86102="/root/firmware/nanokvm_update_86102"
    local bin86102=(/root/firmware/nanokvm_86102*.bin)

    if [[ -x "$update86102" && -f "${bin86102[0]}" ]]; then
        log_info "Updating 86102 firmware..."
        "$update86102" "${bin86102[@]}"
    else
        log_debug "86102 updater or bin not found, skip"
    fi

    local update6911="/root/firmware/nanokvm_update_6911"
    local bin6911=(/root/firmware/nanokvm_6911*.bin)

    if [[ -x "$update6911" && -f "${bin6911[0]}" ]]; then
        log_info "Updating 6911 firmware..."
        "$update6911" "${bin6911[@]}"
    else
        log_debug "6911 updater or bin not found, skip"
    fi

    log_debug "Reloading lt6911_manage module..."
    insmod_lt6911_manage || {
        log_error "Failed to load lt6911_manage"
        return 1
    }

    log_info "HDMI firmware update complete"
    return 0
}

sd_test() {
    log_info "Starting SD test"
    if [ -f /root/factory_test/done/.sd.done ]; then
        log_info "SD test already completed, skipping."
        return 0
    fi
    set +e

    local sd_device="/dev/mmcblk1"

    if [ ! -b "$sd_device" ]; then
        log_error "SD device $sd_device not found"
        return 1
    fi

    touch "/root/factory_test/done/.sd.done"
    log_info "SD test completed successfully"
    set -e

    return 0
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

iperf3_speed_test() {
    local SERVER_IP="$1"
    local PORT="${2:-5201}"
    local DURATION="${3:-5}"
    local RECEIVE="${4:-0}"
    local SPEED="${5:-1000}"
    local BIND_IP="${6:-}"

    log_debug "Starting iperf3 TCP upload test: $DURATION seconds $BIND_IP -> $SERVER_IP:$PORT"
    if [[ -z "$BIND_IP" ]]; then
        output=$(iperf3 -c "$SERVER_IP" -p "$PORT" -t "$DURATION" -P 4 --format m)
    else
        output=$(iperf3 -c "$SERVER_IP" -p "$PORT" -t "$DURATION" -P 4 --format m -B "$BIND_IP")
    fi

    bandwidth_line=$(echo "$output" | grep -Eo '[0-9.]+ Mbits/sec' | tail -1)

    if [[ -z "$bandwidth_line" ]]; then
        log_warn "Could not parse bandwidth from iperf3 output."
        return 1
    fi

    bandwidth_value=$(echo "$bandwidth_line" | grep -Eo '[0-9.]+')

    bandwidth_int=${bandwidth_value%.*}
    is_greater=$(echo "$bandwidth_value >= $SPEED" | bc)

    if [[ "$is_greater" -eq 1 ]]; then
        log_info "Upload speed test passed: ${bandwidth_value} Mbps >= ${SPEED} Mbps"
    else
        log_error "Upload speed test failed: ${bandwidth_value} Mbps < ${SPEED} Mbps"
        return 1
    fi

    if [[ "$RECEIVE" -eq 1 ]]; then
        log_debug "Starting iperf3 TCP download test: $DURATION seconds $BIND_IP -> $SERVER_IP:$PORT"
        if [[ -z "$BIND_IP" ]]; then
            output_receive=$(iperf3 -R -c "$SERVER_IP" -p "$PORT" -t "$DURATION" --format m)
        else
            output_receive=$(iperf3 -R -c "$SERVER_IP" -p "$PORT" -t "$DURATION" --format m -B "$BIND_IP")
        fi

        bandwidth_line_receive=$(echo "$output_receive" | grep -Eo '[0-9.]+ Mbits/sec' | tail -1)

        if [[ -z "$bandwidth_line_receive" ]]; then
            log_warn "Could not parse bandwidth from iperf3 download output."
            return 1
        fi

        bandwidth_value_receive=$(echo "$bandwidth_line_receive" | grep -Eo '[0-9.]+')

        is_greater_receive=$(echo "$bandwidth_value_receive >= $SPEED" | bc)

        if [[ "$is_greater_receive" -eq 1 ]]; then
            log_info "Download speed test passed: ${bandwidth_value_receive} Mbps >= ${SPEED} Mbps"
        else
            log_error "Download speed test failed: ${bandwidth_value_receive} Mbps < ${SPEED} Mbps"
            return 1
        fi
    fi

    return 0
}

uart_test() {
    local uart_num="$1"
    local uart_dev="/dev/ttyS${uart_num}"
    local test_str="NANOKVM_LOOPBACK_TEST"

    if [ ! -e "$uart_dev" ]; then
        log_error "$uart_dev not found. Please check connection"
        return 1
    fi

    log_debug "Start UART loopback $uart_dev (TX ↔ RX)..."

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
        log_info "UART loopback test passed: $uart_dev"
        return 0
    else
        log_error "UART loopback test failed: $uart_dev"
        return 2
    fi
}

gpio_init() {
    log_debug "Init GPIO for control"

    if [ ! -e /sys/class/gpio/gpio35 ]; then
        echo 35 > /sys/class/gpio/export
    fi

    if [ ! -e /sys/class/gpio/gpio7 ]; then
        echo 7 > /sys/class/gpio/export
    fi

    if [ ! -e /sys/class/gpio/gpio74 ]; then
        echo 74 > /sys/class/gpio/export
    fi

    if [ ! -e /sys/class/gpio/gpio75 ]; then
        echo 75 > /sys/class/gpio/export
    fi

    if [ ! -e /sys/class/gpio/gpio0 ]; then
        echo 0 > /sys/class/gpio/export
    fi

    echo out > /sys/class/gpio/gpio7/direction
    echo out > /sys/class/gpio/gpio35/direction
    echo in > /sys/class/gpio/gpio74/direction
    echo in > /sys/class/gpio/gpio75/direction
    echo in > /sys/class/gpio/gpio0/direction

    echo 0 > /sys/class/gpio/gpio7/value
    echo 0 > /sys/class/gpio/gpio35/value
}

#############################################################
#               ATX Test Error Code Bitmap Table            #
#-----------------------------------------------------------#
# Bit | Description               | Key Combo | Bitmask     #
#-----------------------------------------------------------#
#  0  | Round 1: PWRLED HIGH fail |   "0 0"   |    0x01     #
#  1  | Round 1: HDDLED HIGH fail |   "0 0"   |    0x02     #
#  2  | Round 2: PWRLED HIGH fail |   "1 0"   |    0x04     #
#  3  | Round 2: HDDLED HIGH fail |   "1 0"   |    0x08     #
#  4  | Round 3: PWRLED HIGH fail |   "0 1"   |    0x10     #
#  5  | Round 3: HDDLED HIGH fail |   "0 1"   |    0x20     #
#  6  | Round 4: PWRLED LOW fail  |   "1 1"   |    0x40     #
#  7  | Round 4: HDDLED LOW fail  |   "1 1"   |    0x80     #
#############################################################

atx_test() {
    if [ -f /root/factory_test/done/.atx.done ]; then
        log_info "ATX test already completed, skipping."
        return 0
    fi
    set +e

    local mode="$1"
    [ -z "$mode" ] && mode="atx"

    log_debug "Start ATX power control test (mode: $mode)"

    local temp=0
    local keys=("0 0" "1 0" "0 1" "1 1")
    local index=0

    for key in "${keys[@]}"; do
        pwr=$(echo $key | awk '{print $1}')
        rst=$(echo $key | awk '{print $2}')

        echo $pwr > /sys/class/gpio/gpio7/value
        echo $rst > /sys/class/gpio/gpio35/value
        sleep 0.1

        pwr_led=$(cat /sys/class/gpio/gpio75/value)
        [ "$mode" = "atx" ] && hdd_led=$(cat /sys/class/gpio/gpio74/value)
        log_debug "Round $((index+1)) key=($pwr $rst) PWRLED=$pwr_led HDDLED=${hdd_led:-N/A}"

        if [ "$pwr" = "1" ] && [ "$rst" = "1" ]; then
            [ "$pwr_led" != "0" ] && temp=$((temp | (1 << (index*2)))) # PWRLED error
            if [ "$mode" = "atx" ] && [ "$hdd_led" != "0" ]; then
                temp=$((temp | (1 << (index*2 + 1)))) # HDDLED error
            fi
        else
            [ "$pwr_led" != "1" ] && temp=$((temp | (1 << (index*2)))) # PWRLED error
            if [ "$mode" = "atx" ] && [ "$hdd_led" != "1" ]; then
                temp=$((temp | (1 << (index*2 + 1)))) # HDDLED error
            fi
        fi

        index=$((index + 1))
    done

    if [ "$temp" -ne 0 ]; then
        log_error "ATX test failed, temp=0x$(printf "%02X" $temp)"
        send_msg "atx,fail,0x$(printf "%02X" $temp)"
        return 1
    else
        log_info "ATX test passed"
        touch "/root/factory_test/done/.atx.done"
        set -e
        return 0
    fi
}

insmod_lt6911_manage() {
    if lsmod | grep -q '^lt6911_manage'; then
        log_debug "Unloading lt6911_manage module"
        rmmod lt6911_manage
    fi

    local ko_file
    ko_file=$(find ko -name 'lt6911_manage*.ko' | head -n 1)
    if [[ -z "$ko_file" ]]; then
        log_debug "lt6911_manage*.ko module not found in ko/"
        return 1
    fi

    log_info "Loading module $ko_file"
    insmod "$ko_file"
}

insmod_fb() {
    if lsmod | grep -q '^fbtft'; then
        log_info "fbtft module already loaded"
        return 0
    fi

    local base_modules=(fbtft fb_jd9853 rotary_encoder gpio_keys)
    for mod in "${base_modules[@]}"; do
        local files=(ko/${mod}*.ko)
        if [[ -e "${files[0]}" ]]; then
            for file in "${files[@]}"; do
                log_info "Loading module $(basename "$file")"
                insmod "$file"
            done
        else
            log_warn "Optional module ${mod}*.ko not found, skipping"
        fi
    done

    log_info "All display-related modules loaded"
}

write_edid() {
    local edid_path="/proc/lt6911_info/edid"
    if [ ! -e "$edid_path" ]; then
        log_warn "$edid_path not found, waiting 2s..."
        sleep 2
    fi
    if [ ! -e "$edid_path" ]; then
        log_error "$edid_path still not found, exiting."
        return 1
    fi

    local edid_file
    edid_file=$(find firmware -name 'edid*.bin' | head -n 1)

    log_info "Writing $edid_file to $edid_path"
    cat "$edid_file" > "$edid_path"

    log_debug "Reading back EDID from $edid_path for verification"
    local readback_file="/tmp/edid_readback.bin"
    cat "$edid_path" > "$readback_file"
    sleep 1

    local orig_sha readback_sha
    orig_sha=$(sha256sum "$edid_file" | awk '{print $1}')
    readback_sha=$(sha256sum "$readback_file" | awk '{print $1}')

    if [[ "$orig_sha" == "$readback_sha" ]]; then
        log_info "EDID write and readback verification passed"
        log_debug "Original EDID SHA256: $orig_sha"
        log_debug "Readback EDID SHA256: $readback_sha"
    # else
    #     log_error "EDID verification failed! (orig: $orig_sha, readback: $readback_sha)"
    #     return 1
    fi
}

run_vin_bin() {
    if pgrep -f "kvm_vin_test" > /dev/null 2>&1; then
        log_debug "Found existing kvm_vin_test process, killing it..."
        pkill -f kvm_vin_test || true

        local wait_count=0
        while pgrep -f "kvm_vin_test" > /dev/null 2>&1 && [ $wait_count -lt 10 ]; do
            sleep 0.5
            wait_count=$((wait_count + 1))
            log_debug "Waiting for process to terminate... ($wait_count/10)"
        done

        if pgrep -f "kvm_vin_test" > /dev/null 2>&1; then
            log_debug "Process still running, force killing with SIGKILL..."
            pkill -9 -f kvm_vin_test || true
            sleep 1
        else
            log_debug "Process successfully terminated normally"
        fi
    fi

    local vin_test_bin
    vin_test_bin=$(find firmware -type f -name 'kvm_vin_test*' | head -n 1)
    if [[ -z "$vin_test_bin" ]]; then
        log_error "kvm_vin_test* binary not found in firmware/"
        echo ""
        return 0
    fi

    log_debug "Starting kvm_vin_test..."
    chmod +x "$vin_test_bin"

    # Start the process in background and capture PID immediately
    "$vin_test_bin" </dev/null >/dev/null 2>&1 &
    local vin_test_pid=$!

    # Give it a moment to start
    sleep 0.5

    # Check if process is still running
    if ps -p "$vin_test_pid" > /dev/null 2>&1; then
        log_debug "kvm_vin_test is running with PID $vin_test_pid"
        echo "$vin_test_pid"
        return 0
    else
        log_error "kvm_vin_test failed to start or exited immediately"
        echo ""
        return 0
    fi
}

vin_test() {
    local vin_test_pid=$1
    local out_fps
    local retry_count=0
    local max_retries=15

    if [[ -z "$vin_test_pid" ]]; then
        log_error "vin_test: empty PID, skip test"
        return 1
    fi

    while [ $retry_count -lt $max_retries ]; do
        out_fps=$(awk '/^\[CHN\]/ {chn=1; next} chn && NF && $1 ~ /^[0-9]+$/ {print $6; exit}' /proc/ax_proc/vin/statistics)

        if [[ "$out_fps" =~ ^[0-9]+([.][0-9]+)?$ ]] && echo "$out_fps > 0" | bc | grep -q 1; then
            log_info "MIPI image received success, OutFps = $out_fps (attempt $((retry_count + 1)))"
            kill "$vin_test_pid" 2>/dev/null || true
            return 0
        fi

        retry_count=$((retry_count + 1))
        if [ $retry_count -lt $max_retries ]; then
            log_debug "Attempt $retry_count failed, OutFps = $out_fps, retrying in 1 second..."
            sleep 1
        fi
    done

    log_error "MIPI image received failed after $max_retries attempts, OutFps = $out_fps"

    log_info "Running firmware_hdmi to try repairing HDMI..."
    if ! firmware_hdmi; then
        log_error "firmware_hdmi failed, skip retest"
        return 1
    fi

    retry_count=0
    while [ $retry_count -lt $max_retries ]; do
        out_fps=$(awk '/^\[CHN\]/ {chn=1; next} chn && NF && $1 ~ /^[0-9]+$/ {print $6; exit}' /proc/ax_proc/vin/statistics)

        if [[ "$out_fps" =~ ^[0-9]+([.][0-9]+)?$ ]] && echo "$out_fps > 0" | bc | grep -q 1; then
            log_info "MIPI recovered after firmware update! OutFps = $out_fps"
            kill "$new_pid" 2>/dev/null || true
            return 0
        fi

        retry_count=$((retry_count + 1))
        if [ $retry_count -lt $max_retries ]; then
            log_debug "Retry after repair $retry_count failed, OutFps = $out_fps..."
            sleep 1
        fi
    done

    log_error "MIPI image still failed after firmware update, OutFps = $out_fps"
    kill "$new_pid" 2>/dev/null || true
    return 1
}

screen_test() {
    log_debug "Start screen test"

    pkill -f kvm_ui_test || true
    pkill -f kvm_spi_ui_test || true

    local ui_test_bin

    if [ "$DEVICE_TYPE" = "Desk" ]; then
        ui_test_bin=$(find firmware -type f -name 'kvm_spi_ui_test*' | head -n 1)
    else
        ui_test_bin=$(find firmware -type f -name 'kvm_ui_test*' | head -n 1)
    fi

    if [[ -z "$ui_test_bin" ]]; then
        log_error "kvm_ui_test* binary not found in firmware/"
        return 1
    fi

    log_debug "Start UI test with $ui_test_bin"
    chmod +x "$ui_test_bin"
    "$ui_test_bin" &

    local ui_test_pid=$!
    local wait_time=0
    local max_wait=5
    while ! ps -p "$ui_test_pid" > /dev/null; do
        sleep 1
        wait_time=$((wait_time + 1))
        if [ "$wait_time" -ge "$max_wait" ]; then
            log_error "kvm_ui_test can't start"
            return 1
        fi
    done
    log_debug "kvm_ui_test is running with PID $ui_test_pid"

    log_debug "Waiting for GPIO0 button press (low level)..."
    local button_pressed=0
    local check_count=0
    local max_checks=2400

    local initial_gpio0_value=$(cat /sys/class/gpio/gpio0/value)
    if [ "$initial_gpio0_value" = "0" ]; then
        log_error "GPIO0 is already low level before test start, screen test failed"
        kill "$ui_test_pid" 2>/dev/null || true
        return 1
    fi

    log_debug "Initial GPIO0 level is high, starting button press detection"
    while [ $check_count -lt $max_checks ]; do
        local gpio0_value=$(cat /sys/class/gpio/gpio0/value)

        if [ "$gpio0_value" = "0" ]; then
            log_info "GPIO0 button pressed detected, screen test passed"
            button_pressed=1
            break
        fi

        check_count=$((check_count + 1))
        sleep 0.1
    done

    kill "$ui_test_pid" 2>/dev/null || true

    if [ "$button_pressed" = "1" ]; then
        log_info "Screen test completed successfully"
        return 0
    else
        log_error "Screen test timeout: no button press detected after $max_checks seconds"
        return 1
    fi
}

touch_test() {
    log_debug "Start touch test"

    pkill -f kvm_spi_ui_test || true

    local ui_test_bin

    ui_test_bin=$(find firmware -type f -name 'kvm_spi_ui_test*' | head -n 1)

    if [[ -z "$ui_test_bin" ]]; then
        log_error "kvm_spi_ui_test* binary not found in firmware/"
        return 1
    fi

    log_debug "Start Touch test with $ui_test_bin"
    chmod +x "$ui_test_bin"
    "$ui_test_bin" --touch &

    local ui_test_pid=$!
    local wait_time=0
    local max_wait=5
    while ! ps -p "$ui_test_pid" > /dev/null; do
        sleep 1
        wait_time=$((wait_time + 1))
        if [ "$wait_time" -ge "$max_wait" ]; then
            log_error "kvm_ui_test can't start"
            return 1
        fi
    done
    log_debug "kvm_ui_test is running with PID $ui_test_pid"

    local timeout=240
    local elapsed=0
    while kill -0 "$ui_test_pid" 2>/dev/null; do
        sleep 1
        elapsed=$((elapsed + 1))
        if [ "$elapsed" -ge "$timeout" ]; then
            log_error "touch test timeout, killing process $ui_test_pid"
            kill -9 "$ui_test_pid" 2>/dev/null || true
            return 1
        fi
    done

    return 0
}

rotary_test() {
    log_debug "Start rotary test"

    pkill -f kvm_spi_ui_test || true

    local ui_test_bin

    ui_test_bin=$(find firmware -type f -name 'kvm_spi_ui_test*' | head -n 1)

    if [[ -z "$ui_test_bin" ]]; then
        log_error "kvm_spi_ui_test* binary not found in firmware/"
        return 1
    fi

    log_debug "Start rotary test with $ui_test_bin"
    chmod +x "$ui_test_bin"
    "$ui_test_bin" --rotary &

    local ui_test_pid=$!
    local wait_time=0
    local max_wait=5
    while ! ps -p "$ui_test_pid" > /dev/null; do
        sleep 1
        wait_time=$((wait_time + 1))
        if [ "$wait_time" -ge "$max_wait" ]; then
            log_error "kvm_ui_test can't start"
            return 1
        fi
    done
    log_debug "kvm_ui_test is running with PID $ui_test_pid"

    local timeout=240
    local elapsed=0
    while kill -0 "$ui_test_pid" 2>/dev/null; do
        sleep 1
        elapsed=$((elapsed + 1))
        if [ "$elapsed" -ge "$timeout" ]; then
            log_error "rotary test timeout, killing process $ui_test_pid"
            kill -9 "$ui_test_pid" 2>/dev/null || true
            return 1
        fi
    done

    return 0
}

check_wifi_interface() {
    local wifi_interface="${1:-wlan0}"

    log_debug "Checking if WiFi interface $wifi_interface exists"

    if [ -e "/sys/class/net/$wifi_interface" ]; then
        log_info "WiFi interface $wifi_interface found"
        return 0
    else
        log_info "WiFi interface $wifi_interface not found - WiFi functionality disabled"
        return 1
    fi
}

wifi_test() {
    log_debug "Start WiFi test"

    local wifi_ssid="$HOST_WIFI_SSID"
    local wifi_password="12345678"
    local wifi_interface="wlan0"

    log_info "Connecting to WiFi: $wifi_ssid"

    ip link set dev "$wifi_interface" up
    sleep 2

    local wpa_config="/tmp/wpa_supplicant.conf"
    cat > "$wpa_config" << EOF
ctrl_interface=/var/run/wpa_supplicant
update_config=1
country=CN

network={
    ssid="$wifi_ssid"
    psk="$wifi_password"
    key_mgmt=WPA-PSK
}
EOF

    pkill -f wpa_supplicant || true
    sleep 1

    log_debug "Starting wpa_supplicant"

    local max_retries=2
    local attempt=0
    local connected=0

    while [ $attempt -le $max_retries ]; do
        attempt=$((attempt + 1))
        log_info "WiFi connection attempt $attempt"

        pkill -f wpa_supplicant || true
        rm -f /var/run/wpa_supplicant/"$wifi_interface"
        sleep 1
        wpa_supplicant -B -i "$wifi_interface" -c "$wpa_config" -P /tmp/wpa_supplicant.pid

        local connect_timeout=10
        connected=0
        for ((i=1; i<=connect_timeout; i++)); do
            if wpa_cli -i "$wifi_interface" status | grep -q "wpa_state=COMPLETED"; then
                log_info "WiFi connected successfully"
                connected=1
                break
            fi
            log_debug "Waiting for WiFi connection... ($i/$connect_timeout)"
            sleep 1
        done

        if [ "$connected" -eq 1 ]; then
            break
        else
            log_warn "WiFi connection attempt $attempt failed"
        fi
    done

    if [ "$connected" -eq 0 ]; then
        log_error "Failed to connect to WiFi after $((max_retries+1)) attempts"
        pkill -f wpa_supplicant || true
        return 1
    fi

    log_debug "Requesting IP address via DHCP"
    dhclient -r "$wifi_interface" || true
    dhclient "$wifi_interface"
    sleep 3

    local wifi_ip
    wifi_ip=$(ip -4 addr show "$wifi_interface" | grep -oP '(?<=inet\s)\d+(\.\d+){3}' | head -n1)

    if [ -z "$wifi_ip" ]; then
        log_error "Failed to get IP address on WiFi interface"
        pkill -f wpa_supplicant || true
        return 1
    fi

    log_info "WiFi IP address: $wifi_ip"

    log_info "Starting WiFi speed test (upload only)"
    local wifi_speed_result=0

    if iperf3_speed_test 192.168.4.1 5201 5 0 8 "$wifi_ip"; then
        log_info "WiFi speed test passed"
    else
        log_error "WiFi speed test failed, retrying..."
        if iperf3_speed_test 192.168.4.1 5201 5 0 8 "$wifi_ip"; then
            log_info "WiFi speed test passed on retry"
        else
            log_error "WiFi speed test failed on retry"
            wifi_speed_result=1
        fi
    fi

    log_debug "Cleaning up WiFi connection"
    ip addr flush dev "$wifi_interface" || true
    pkill -f wpa_supplicant || true
    ip link set dev "$wifi_interface" down
    rm -f "$wpa_config"

    if [ "$wifi_speed_result" -eq 0 ]; then
        log_info "WiFi test completed successfully"
        send_msg "wifi,done"
        return 0
    else
        return 1
    fi
}

wifi_stop() {
    log_debug "Stopping WiFi services"

    local wifi_interface="wlan0"
    pkill -f "dhclient" || true
    pkill -f "wpa_supplicant" || true
    ip addr flush dev "$wifi_interface" 2>/dev/null || true
    ip route del default via 192.168.4.1 dev "$wifi_interface" 2>/dev/null || true
    ip link set dev "$wifi_interface" down 2>/dev/null || true
    rm -f /tmp/wpa_supplicant.conf
    rm -f /tmp/wpa_supplicant.pid

    log_info "WiFi services stopped and cleaned up"
}

wifi_ap_start() {
    log_debug "Starting WiFi AP mode"

    local wifi_interface="${1:-wlan0}"
    local ap_ssid="${2:-$(hostname)}"
    local ap_password="${3:-12345678}"
    local ap_ip="${4:-192.168.4.1}"

    if [ ! -e "/sys/class/net/$wifi_interface" ]; then
        log_error "WiFi interface $wifi_interface not found"
        return 1
    fi

    pkill -f "wpa_supplicant" || true
    pkill -f "hostapd" || true
    pkill -f "dhclient" || true
    sleep 1

    ip link set dev "$wifi_interface" up
    ip addr flush dev "$wifi_interface"
    ip addr add "$ap_ip/24" dev "$wifi_interface"

    local hostapd_conf="/tmp/hostapd.conf"
    cat > "$hostapd_conf" << EOF
interface=$wifi_interface
driver=nl80211
ssid=$ap_ssid
hw_mode=g
channel=7
wmm_enabled=0
macaddr_acl=0
auth_algs=1
ignore_broadcast_ssid=0
wpa=2
wpa_passphrase=$ap_password
wpa_key_mgmt=WPA-PSK
wpa_pairwise=TKIP
rsn_pairwise=CCMP
EOF

    log_debug "Starting hostapd with configuration: $hostapd_conf"
    hostapd -B "$hostapd_conf" -P /tmp/hostapd.pid

    local wait_count=0
    local max_wait=10
    while [ $wait_count -lt $max_wait ]; do
        if pgrep -f "hostapd" >/dev/null 2>&1; then
            log_info "WiFi AP started successfully"
            log_info "  SSID: $ap_ssid"
            log_info "  Password: $ap_password"
            log_info "  IP: $ap_ip"
            send_msg "ap_ssid,$ap_ssid"
            return 0
        fi
        sleep 1
        wait_count=$((wait_count + 1))
    done

    log_error "Failed to start WiFi AP mode"
    return 1
}

wifi_ap_stop() {
    log_debug "Stopping WiFi AP mode"

    local wifi_interface="${1:-wlan0}"
    pkill -f "hostapd" || true
    rm -f /tmp/hostapd.conf
    rm -f /tmp/hostapd.pid
    ip addr flush dev "$wifi_interface" 2>/dev/null || true
    ip link set dev "$wifi_interface" down 2>/dev/null || true
    log_info "WiFi AP mode stopped and cleaned up"
    return 0
}

check_wifi_mode_stop() {
    log_debug "Checking WiFi mode"

    local wifi_interface="${1:-wlan0}"
    local mode="unknown"

    if [ ! -e "/sys/class/net/$wifi_interface" ]; then
        log_error "WiFi interface $wifi_interface not found"
        echo "not_found"
        return 1
    fi

    if [ "$mode" = "unknown" ]; then
        if pgrep -f "hostapd" >/dev/null 2>&1; then
            mode="ap"
        elif pgrep -f "wpa_supplicant" >/dev/null 2>&1; then
            mode="station"
        fi
    fi

    log_info "WiFi interface $wifi_interface is in mode: $mode"
    echo "$mode"

    case "$mode" in
        "ap")
            log_debug "WiFi is in AP mode, calling wifi_ap_stop"
            wifi_ap_stop
            return 0
            ;;
        "station")
            log_debug "WiFi is in station mode, calling wifi_stop"
            wifi_stop
            return 0
            ;;
        *)
            log_debug "WiFi mode unknown or not active"
            return 0
            ;;
    esac
}

wifi_ap_test() {
    log_debug "Start WiFi AP test"
    local timeout_sec="${1:-20}"
    local listen_port="${2:-2334}"
    local received_msg raw_msg

    raw_msg=$(get_msg "$listen_port" "$timeout_sec")
    received_msg=$(echo "$raw_msg" | grep -Eo 'ap_ok' | head -n 1)

    if [ -n "$received_msg" ]; then
        log_debug "Received message: '$received_msg'"
        if [ "$received_msg" = "ap_ok" ]; then
            log_info "WiFi AP test passed - received confirmation message"
            return 0
        else
            log_error "WiFi AP test: Received unexpected message: '$received_msg' (expected 'ap_ok')"
            return 1
        fi
    else
        log_error "WiFi AP test: No message received within $timeout_sec seconds"
        return 1
    fi
}

eth_test() {
    if [ -f /root/factory_test/done/.eth.done ]; then
        log_info "eth test already completed, skipping."
        return 0
    fi

    set +e

    iperf3_speed_test $HOST_IP 5201 3 0 700
    local result=$?

    if [[ $result -ne 0 ]]; then
        log_error "iperf3 speed test failed, retrying..."
        iperf3_speed_test $HOST_IP 5201 3 0 700
        if [[ $? -ne 0 ]]; then
            log_error "iperf3 speed test failed again, exiting."
            return 1
        fi
    fi

    touch "/root/factory_test/done/.eth.done"
    set -e
    return 0
}

uart_test_all() {
    if [ -f /root/factory_test/done/.uart.done ]; then
        log_info "UART test already completed, skipping."
        return 0
    fi

    set +e

    uart_test 1
    uart1_result=$?
    uart_test 2
    uart2_result=$?

    if [[ $uart1_result -ne 0 || $uart2_result -ne 0 ]]; then
        log_error "UART test failed (UART1: $uart1_result, UART2: $uart2_result), exiting."
        return 1
    fi

    touch "/root/factory_test/done/.uart.done"
    set -e
    return 0
}

report_completed_tests
gpio_init

pkill -f kvm_ui_test || true
pkill -f kvm_spi_ui_test || true

VIN_PID=""
if [ ! -f /root/factory_test/done/.mipi.done ]; then
    if VIN_PID=$(run_vin_bin); then
        log_debug "VIN_PID obtained: $VIN_PID"
    else
        log_error "Failed to start vin binary"
        VIN_PID=""
    fi
fi

insmod_fb
devmem 0x0230000C 32 0x00060003
sleep 1

send_msg "screen,start"
CATEGORY="screen"
trap fail_handler ERR
if [ ! -f /root/factory_test/done/.screen.done ]; then
    screen_test
fi
# touch "/root/factory_test/done/.screen.done"
send_msg "screen,done"

send_msg "touch,start"
CATEGORY="touch"
trap fail_handler ERR
if [ "$DEVICE_TYPE" = "Desk" ] && [ ! -f /root/factory_test/done/.touch.done ]; then
    touch_test
fi
# touch "/root/factory_test/done/.touch.done"
send_msg "touch,done"

send_msg "rotary,start"
CATEGORY="rotary"
trap fail_handler ERR
if [ "$DEVICE_TYPE" = "Desk" ] && [ ! -f /root/factory_test/done/.rotary.done ]; then
    rotary_test
fi
# touch "/root/factory_test/done/.rotary.done"
send_msg "rotary,done"

SD_PID=""
if [ "$DEVICE_TYPE" = "Desk" ]; then
    send_msg "sd,start"
    sd_test &
    SD_PID=$!
fi

if [ "$DEVICE_TYPE" = "ATX" ]; then
    send_msg "emmc,start"
fi
emmc_test &
EMMC_PID=$!

eth_test &
ETH_PID=$!

UART_PID=""
if [ "$DEVICE_TYPE" = "Desk" ]; then
    uart_test_all &
    UART_PID=$!
fi

ATX_PID=""
if [ "$DEVICE_TYPE" = "Desk" ]; then
    atx_test desk &
    ATX_PID=$!
elif [ "$DEVICE_TYPE" = "ATX" ]; then
    atx_test atx &
    ATX_PID=$!
fi

if [ -n "$SD_PID" ]; then
    log_debug "Waiting for SD test to complete (PID: $SD_PID)..."
    wait $SD_PID || true
    log_debug "SD test completed"
fi

log_debug "Waiting for eMMC test to complete (PID: $EMMC_PID)..."
wait $EMMC_PID || true
log_debug "eMMC test completed"

log_debug "Waiting for Ethernet test to complete (PID: $ETH_PID)..."
wait $ETH_PID || true
log_debug "Ethernet test completed"

if [ -n "$UART_PID" ]; then
    log_debug "Waiting for UART test to complete (PID: $UART_PID)..."
    wait $UART_PID || true
    log_debug "UART test completed"
fi

if [ -n "$ATX_PID" ]; then
    log_debug "Waiting for ATX test to complete (PID: $ATX_PID)..."
    wait $ATX_PID || true
    log_debug "ATX test completed"
fi

log_debug "All background tests completed, proceeding with verification..."

send_msg "sd,start"
CATEGORY="sd"
trap fail_handler ERR
if [ "$DEVICE_TYPE" = "Desk" ] && [ ! -f /root/factory_test/done/.sd.done ]; then
    log_error "SD test failed, exiting"
    fail_handler
fi
send_msg "sd,done"

send_msg "emmc,start"
CATEGORY="emmc"
trap fail_handler ERR
if [ ! -f /root/factory_test/done/.emmc.done ]; then
    log_error "emmc test failed, exiting"
    fail_handler
fi
send_msg "emmc,done"

send_msg "eth,start"
CATEGORY="eth"
trap fail_handler ERR
if [ ! -f /root/factory_test/done/.eth.done ]; then
    log_error "eth test failed, exiting"
    fail_handler
fi
send_msg "eth,done"

send_msg "uart,start"
CATEGORY="uart"
trap fail_handler ERR
if [ "$DEVICE_TYPE" = "Desk" ] && [ ! -f /root/factory_test/done/.uart.done ]; then
    log_error "UART test failed, exiting"
    fail_handler
fi
send_msg "uart,done"

send_msg "atx,start"
CATEGORY="atx"
trap fail_handler ERR
if [ ! -f /root/factory_test/done/.atx.done ]; then
    log_error "ATX test failed, exiting"
    fail_handler
fi
send_msg "atx,done"

send_msg "mipi,start"
CATEGORY="mipi"
trap fail_handler ERR
if [ ! -f /root/factory_test/done/.mipi.done ]; then
    vin_test "$VIN_PID"
fi
touch "/root/factory_test/done/.mipi.done"
send_msg "mipi,done"

if check_wifi_interface; then
    check_wifi_mode_stop
fi

send_msg "wifi,start"
CATEGORY="wifi"
trap fail_handler ERR
if [ ! -f /root/factory_test/done/.wifi.done ]; then
    if check_wifi_interface; then
        wifi_test
    else
        log_info "Skipping WiFi test - no WiFi interface found"
    fi
fi
touch "/root/factory_test/done/.wifi.done"
send_msg "wifi,done"

if check_wifi_interface; then
    check_wifi_mode_stop
fi

send_msg "ap,start"
CATEGORY="ap"
trap fail_handler ERR
if [ ! -f /root/factory_test/done/.ap.done ]; then
    if check_wifi_interface; then
        wifi_stop
        wifi_ap_start
        wifi_ap_test
    else
        log_info "Skipping WiFi AP test - no WiFi interface found"
    fi
fi
touch "/root/factory_test/done/.ap.done"
send_msg "ap,done"

exit 0