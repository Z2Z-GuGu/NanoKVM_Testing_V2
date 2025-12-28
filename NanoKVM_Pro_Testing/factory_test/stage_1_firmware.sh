#!/bin/bash

LOGFILE="/tmp/stage_1_firmware.log"
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

fail_handler() {
    log_error "Firmware update failed!"
    send_msg "firmware,fail"
    exit 1
}

trap fail_handler ERR

firmware_dir="/root/firmware"
UBOOT_BIN=$(ls $firmware_dir/u-boot_signed*.bin | head -n1)
DTB_BIN=$(ls $firmware_dir/AX630C_emmc_arm64_k419_sipeed_nanokvm_signed*.dtb | head -n1)
KERNEL_BIN=$(ls $firmware_dir/boot_signed*.bin | head -n1)

UBOOT_DEV1="/dev/mmcblk0p5"
UBOOT_DEV2="/dev/mmcblk0p6"
DTB_DEV1="/dev/mmcblk0p12"
DTB_DEV2="/dev/mmcblk0p13"
KERNEL_DEV1="/dev/mmcblk0p14"
KERNEL_DEV2="/dev/mmcblk0p15"

send_msg "firmware,start"

set_mac_addr() {
    mac_uid=$(sha512sum /device_key  | head -c 4)
    mac_hi=$(echo $mac_uid | cut -c 1-2)
    mac_lo=$(echo $mac_uid | cut -c 3-4)
    new_mac="48:da:35:6d:${mac_hi}:${mac_lo}"
    interfaces_file="/etc/network/interfaces"
    if grep -q "hwaddress ether" "$interfaces_file"; then
        sed -i "s/^\(.*hwaddress ether \)[^ ]*\$/\1$new_mac/" "$interfaces_file"
    else
        echo "hwaddress ether $new_mac" >> "$interfaces_file"
    fi
    # hostname
    echo kvm-${mac_hi}${mac_lo} > /etc/hostname
    /usr/bin/hostname kvm-${mac_hi}${mac_lo}
    echo "New MAC address set to $new_mac"
    echo "Hostname set to kvm-${mac_hi}${mac_lo}"
    sync
}

remove_file() {
    local remove_file_list="/root/remove_file.txt"

    if [ ! -f "$remove_file_list" ]; then
        log_warn "Remove file list not found: $remove_file_list"
        return 0
    fi

    log_step "Removing unwanted files..."
    local removed_count=0
    local total_count=0

    while IFS= read -r file_path; do
        [[ -z "$file_path" || "$file_path" =~ ^[[:space:]]*# ]] && continue

        total_count=$((total_count + 1))

        if [ -e "$file_path" ]; then
            if rm -rf "$file_path" 2>/dev/null; then
                log_debug "Removed: $file_path"
                removed_count=$((removed_count + 1))
            else
                log_warn "Failed to remove: $file_path"
            fi
        else
            log_debug "File not found (skipping): $file_path"
        fi
    done < "$remove_file_list"

    log_info "✅ Cleanup completed: $removed_count/$total_count files removed"
}

verify_sha256() {
    log_step "Verifying firmware SHA256 checksums..."

    local sha256_file="$firmware_dir/sha256.txt"
    if [ ! -f "$sha256_file" ]; then
        log_error "Missing SHA256 checksum file: $sha256_file"
        exit 1
    fi

    pushd "$firmware_dir" > /dev/null

    while read -r hash full_path filename; do
        if [ ! -f "$filename" ]; then
            log_error "Missing file: $filename"
            exit 1
        fi

        actual_hash=$(sha256sum "$filename" | awk '{print $1}')
        if [ "$actual_hash" != "$hash" ]; then
            log_error "Checksum mismatch for $filename!"
            echo "Expected: $hash"
            echo "Actual  : $actual_hash"
            exit 1
        else
            log_info "✅ Verified checksum for $filename"
        fi
    done < "$sha256_file"

    popd > /dev/null
}

set_mac_addr

if [ ! -e /usr/sbin/ether-wake ]; then
    ln -sf /usr/sbin/etherwake /usr/sbin/ether-wake
    echo "Created symlink: /usr/sbin/ether-wake -> /usr/sbin/etherwake"
else
    echo "/usr/sbin/ether-wake already exists, skipping"
fi

mkdir -p /data

verify_sha256

remove_file

log_step "Writing U-Boot to eMMC partitions..."
dd if="$UBOOT_BIN" of="$UBOOT_DEV1" bs=4K conv=notrunc status=none && log_info "✅ Written $UBOOT_DEV1 successfully"
dd if="$UBOOT_BIN" of="$UBOOT_DEV2" bs=4K conv=notrunc status=none && log_info "✅ Written $UBOOT_DEV2 successfully"

log_step "Writing DTB to eMMC partitions..."
dd if="$DTB_BIN" of="$DTB_DEV1" bs=4K conv=notrunc status=none && log_info "✅ Written $DTB_DEV1 successfully"
dd if="$DTB_BIN" of="$DTB_DEV2" bs=4K conv=notrunc status=none && log_info "✅ Written $DTB_DEV2 successfully"

log_step "Writing Kernel to eMMC partitions..."
dd if="$KERNEL_BIN" of="$KERNEL_DEV1" bs=4K conv=notrunc status=none && log_info "✅ Written $KERNEL_DEV1 successfully"
dd if="$KERNEL_BIN" of="$KERNEL_DEV2" bs=4K conv=notrunc status=none && log_info "✅ Written $KERNEL_DEV2 successfully\n"

rsync -aq --exclude=boot/ overlay/ /
# rsync -aq overlay/boot / --no-owner --no-group

sync
sync
sync

echo "Firmware update completed. System will reboot now."
echo "__STAGE_1_OK__"
touch "/root/factory_test/done/.stage_1_firmware.sh.done"
sync
sync
sync
reboot

exit 0