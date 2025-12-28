#!/bin/bash

set -e
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOGFILE="/tmp/factory_test.log"
STAGE_FILES=($(ls "$SCRIPT_DIR"/stage_*.sh | sort))
TOTAL=${#STAGE_FILES[@]}
PASS_COUNT=0
FAIL_COUNT=0
FORCE=${FORCE:-0}

log_info()  { echo -e "\e[32m[INFO] $*\e[0m"; }
log_warn()  { echo -e "\e[33m[WARN] $*\e[0m"; }
log_error() { echo -e "\e[31m[ERROR] $*\e[0m"; }
log_step()  { echo -e "\e[1;34m===> $*\e[0m"; }
log_hr()    { echo -e "\033[1;34m----------------------------------------\033[0m"; }
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

exec > >(tee -a "$LOGFILE") 2>&1

get_device_uid() {
    local DEVICE_KEY_FILE="/device_key"
    if [[ ! -f "$DEVICE_KEY_FILE" ]]; then
        echo "Device key file not found: $DEVICE_KEY_FILE" >&2
        return 1
    fi
    DEVICE_UID="$(cat "$DEVICE_KEY_FILE")"
    if [[ -z "$DEVICE_UID" ]]; then
        log_error "Failed to retrieve device UID."
        exit 1
    else
        log_info "Device UID: $DEVICE_UID"
    fi
}

get_device_model() {
    local MODEL_FILE="/proc/device-tree/model"
    if [[ ! -f "$MODEL_FILE" ]]; then
        log_error "Device model file not found: $MODEL_FILE"
        return 1
    fi

    local DEVICE_MODEL="$(tr -d '\0' < "$MODEL_FILE")"
    if [[ -z "$DEVICE_MODEL" ]]; then
        log_error "Failed to retrieve device model."
        return 1
    else
        echo "$DEVICE_MODEL"
        return 0
    fi
}

echo -e "\n===== NanoKVM Test start ====="

mkdir -p "$SCRIPT_DIR/done"
mkdir -p "/root/log"

for i in "${!STAGE_FILES[@]}"; do
    STAGE_SCRIPT="${STAGE_FILES[$i]}"
    STAGE_NAME="$(basename "$STAGE_SCRIPT")"
    STAGE_NUM=$((i+1))
    DONE_FILE="$SCRIPT_DIR/done/.${STAGE_NAME}.done"

    echo -e "\n stage $STAGE_NUM/$TOTAL: $STAGE_NAME"
    log_hr

    DEVICE_MODEL=$(get_device_model)
    log_debug "Detected device model: $DEVICE_MODEL"
    if [ "$STAGE_NAME" == "stage_1_firmware.sh" ] && [ "$DEVICE_MODEL" = "NanoKVM-Pro" ]; then
        touch "$SCRIPT_DIR/done/.stage_1_firmware.sh.done"
        sync
    fi

    if [ "$FORCE" -eq 0 ] && [ -f "$DONE_FILE" ]; then

        if [ "$STAGE_NAME" == "stage_1_firmware.sh" ]; then
            send_msg "firmware,done"
        fi

        log_info "stage $STAGE_NUM: $STAGE_NAME has been completed, skipping"
        let PASS_COUNT=PASS_COUNT+1
        continue
    fi

    bash "$STAGE_SCRIPT"
    RESULT=$?

    if [ "$RESULT" -eq 0 ]; then
        log_info "stage $STAGE_NUM: $STAGE_NAME success"
        echo
        # touch "$DONE_FILE"
        let PASS_COUNT=PASS_COUNT+1
    else
        log_error "stage $STAGE_NUM: $STAGE_NAME fail"
        let FAIL_COUNT=FAIL_COUNT+1
    fi

    if [ "$FAIL_COUNT" -eq 0 ]; then
        log_info "All stages passed, will get device UID"
        get_device_uid
    fi

done
