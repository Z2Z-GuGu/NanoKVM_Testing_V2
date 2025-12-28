#!/bin/bash

LOGFILE="/root/log/app.log"
exec > >(tee -a "$LOGFILE") 2>&1

set -e

get_timestamp() {
    date '+[%Y-%m-%d %H:%M:%S]'
}

log_info()  { echo -e "$(get_timestamp) \e[32m[INFO] $*\e[0m"; }
log_warn()  { echo -e "$(get_timestamp) \e[33m[WARN] $*\e[0m"; }
log_error() { echo -e "$(get_timestamp) \e[31m[ERROR] $*\e[0m"; }
log_step()  { echo -e "$(get_timestamp) \e[1;34m===> $*\e[0m"; }
log_debug() {
    if [ "$DEBUG" == "True" ]; then
        echo -e "$(get_timestamp) \e[1;36m[DEBUG] $*\e[0m"
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

CATEGORY="app"

fail_handler() {
    log_error "$CATEGORY update failed!"
    send_msg "${CATEGORY},fail"
    exit 1
}

trap fail_handler ERR

install_app() {
    local pkg=$(dpkg -l | rg kvm)

    if pgrep -f app_stage_1.sh >/dev/null; then
        log_info "app_stage_1.sh is running, waiting for it to finish..."
        local timeout=120
        local elapsed=0

        while pgrep -f app_stage_1.sh >/dev/null && [ $elapsed -lt $timeout ]; do
            sleep 1
            elapsed=$((elapsed + 1))
        done

        if pgrep -f app_stage_1.sh >/dev/null; then
            log_error "app_stage_1.sh still running after ${timeout}s, force killing..."
            pkill -f app_stage_1.sh
            sleep 2
            if pgrep -f app_stage_1.sh >/dev/null; then
                log_error "app_stage_1.sh did not exit, force kill -9"
                pkill -9 -f app_stage_1.sh
            fi
        else
            log_info "app_stage_1.sh finished after ${elapsed}s"
        fi
    fi

    if [ -z "$pkg" ]; then
        log_info "kvm not installed"
        return -1
    fi

    local status=$(echo "$pkg" | awk '{print $1}')

    if echo "$status" | grep -vq '^ii$'; then
        log_info "kvm installation incomplete"
        return -1
    else
        log_info "systemctl enable kvm service"

        pkill -f kvm

        log_debug "Waiting for all kvm processes to exit..."
        local timeout=10
        local elapsed=0

        while pgrep -f kvm >/dev/null && [ $elapsed -lt $timeout ]; do
            sleep 1
            elapsed=$((elapsed + 1))
        done

        if pgrep -f kvm >/dev/null; then
            log_warn "Some kvm processes still running after ${timeout}s, force killing..."
            pkill -9 -f kvm
            sleep 2
        else
            log_debug "All kvm processes exited after ${elapsed}s"
        fi

        systemctl enable kvmcomm.service
        systemctl start kvmcomm.service
    fi
}

send_msg "app,start"
trap fail_handler ERR
if [ ! -f /root/factory_test/done/.app.done ]; then
    install_app
    sync
    sync
    sync
fi
# touch "/root/factory_test/done/.app.done"
send_msg "app,done"

exit 0