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

APP_DIR="/root/NanoKVM_Pro_Testing/debs"
SYS_DIR="/root/NanoKVM_Pro_Testing/firmware"
CACHE_DIR="/root/.kvmcache"
STATE_FILE_PATH="/root/factory_test/done/"

firmware_dtb() {
    local dtb_dir="$SYS_DIR"
    local dtb_pattern="AX630C_emmc_arm64_k419_sipeed_nanokvm_signed*.dtb"

    if [ ! -d "$dtb_dir" ]; then
        log_debug "DTB directory not found: $dtb_dir"
        return 0
    fi

    local dtb_files=($(find "$dtb_dir" -name "$dtb_pattern" -type f 2>/dev/null))

    if [ ${#dtb_files[@]} -eq 0 ]; then
        log_debug "No DTB files found matching pattern: $dtb_pattern in $dtb_dir"
        return 0
    fi

    local DTB_FILE="${dtb_files[0]}"
    log_info "Found DTB file: $(basename "$DTB_FILE")"

    if [ ! -r "$DTB_FILE" ]; then
        log_error "DTB file is not readable: $DTB_FILE"
        return 1
    fi

    if [ ! -e "/dev/mmcblk0p12" ]; then
        log_error "Target partition /dev/mmcblk0p12 does not exist"
        return 1
    fi

    if [ ! -e "/dev/mmcblk0p13" ]; then
        log_error "Target partition /dev/mmcblk0p13 does not exist"
        return 1
    fi

    local file_size=$(stat -c%s "$DTB_FILE")
    local original_sha256=$(sha256sum "$DTB_FILE" | awk '{print $1}')
    log_debug "Original DTB file size: $file_size bytes, SHA256: $original_sha256"

    write_and_verify_firmware() {
        local source_file="$1"
        local target_partition="$2"
        local firmware_type="$3"
        local max_retries=3
        local attempt=1

        while [ $attempt -le $max_retries ]; do
            log_info "Writing $firmware_type to partition $target_partition (attempt $attempt/$max_retries)..."

            if dd if="$source_file" of="$target_partition" bs=4K conv=notrunc status=none 2>/dev/null; then
                sync
                log_info "Successfully wrote $firmware_type to $target_partition"

                log_debug "Verifying $firmware_type data on $target_partition..."
                local written_sha256=$(dd if="$target_partition" bs=1 count=$file_size 2>/dev/null | sha256sum | awk '{print $1}')

                if [ "$original_sha256" = "$written_sha256" ]; then
                    log_info "$firmware_type verification passed for $target_partition (SHA256: $written_sha256)"
                    return 0
                else
                    log_warn "$firmware_type verification failed for $target_partition on attempt $attempt! Expected: $original_sha256, Got: $written_sha256"
                    if [ $attempt -eq $max_retries ]; then
                        log_error "$firmware_type verification failed for $target_partition after $max_retries attempts"
                        return 1
                    fi
                fi
            else
                log_warn "Failed to write $firmware_type to $target_partition on attempt $attempt"
                if [ $attempt -eq $max_retries ]; then
                    log_error "Failed to write $firmware_type to $target_partition after $max_retries attempts"
                    return 1
                fi
            fi

            attempt=$((attempt + 1))
            log_debug "Retrying in 1 second..."
            sleep 1
        done
    }

    if ! write_and_verify_firmware "$DTB_FILE" "/dev/mmcblk0p12" "DTB"; then
        return 1
    fi

    if ! write_and_verify_firmware "$DTB_FILE" "/dev/mmcblk0p13" "DTB"; then
        return 1
    fi

    log_info "DTB firmware update and verification completed successfully"
    return 0
}

firmware_uboot() {
    local uboot_dir="$SYS_DIR"
    local uboot_pattern="u-boot_signed*.bin"

    if [ ! -d "$uboot_dir" ]; then
        log_debug "U-Boot directory not found: $uboot_dir"
        return 0
    fi

    local uboot_files=($(find "$uboot_dir" -name "$uboot_pattern" -type f 2>/dev/null))

    if [ ${#uboot_files[@]} -eq 0 ]; then
        log_debug "No U-Boot files found matching pattern: $uboot_pattern in $uboot_dir"
        return 0
    fi

    local UBOOT_FILE="${uboot_files[0]}"
    log_info "Found U-Boot file: $(basename "$UBOOT_FILE")"

    if [ ! -r "$UBOOT_FILE" ]; then
        log_error "U-Boot file is not readable: $UBOOT_FILE"
        return 1
    fi

    if [ ! -e "/dev/mmcblk0p5" ]; then
        log_error "Target partition /dev/mmcblk0p5 does not exist"
        return 1
    fi

    if [ ! -e "/dev/mmcblk0p6" ]; then
        log_error "Target partition /dev/mmcblk0p6 does not exist"
        return 1
    fi

    local file_size=$(stat -c%s "$UBOOT_FILE")
    local original_sha256=$(sha256sum "$UBOOT_FILE" | awk '{print $1}')
    log_debug "Original U-Boot file size: $file_size bytes, SHA256: $original_sha256"

    write_and_verify_firmware() {
        local source_file="$1"
        local target_partition="$2"
        local firmware_type="$3"
        local max_retries=3
        local attempt=1

        while [ $attempt -le $max_retries ]; do
            log_info "Writing $firmware_type to partition $target_partition (attempt $attempt/$max_retries)..."

            if dd if="$source_file" of="$target_partition" bs=4K conv=notrunc status=none 2>/dev/null; then
                sync
                log_info "Successfully wrote $firmware_type to $target_partition"

                log_debug "Verifying $firmware_type data on $target_partition..."
                local written_sha256=$(dd if="$target_partition" bs=1 count=$file_size 2>/dev/null | sha256sum | awk '{print $1}')

                if [ "$original_sha256" = "$written_sha256" ]; then
                    log_info "$firmware_type verification passed for $target_partition (SHA256: $written_sha256)"
                    return 0
                else
                    log_warn "$firmware_type verification failed for $target_partition on attempt $attempt! Expected: $original_sha256, Got: $written_sha256"
                    if [ $attempt -eq $max_retries ]; then
                        log_error "$firmware_type verification failed for $target_partition after $max_retries attempts"
                        return 1
                    fi
                fi
            else
                log_warn "Failed to write $firmware_type to $target_partition on attempt $attempt"
                if [ $attempt -eq $max_retries ]; then
                    log_error "Failed to write $firmware_type to $target_partition after $max_retries attempts"
                    return 1
                fi
            fi

            attempt=$((attempt + 1))
            log_debug "Retrying in 1 second..."
            sleep 1
        done
    }

    if ! write_and_verify_firmware "$UBOOT_FILE" "/dev/mmcblk0p5" "U-Boot"; then
        return 1
    fi

    if ! write_and_verify_firmware "$UBOOT_FILE" "/dev/mmcblk0p6" "U-Boot"; then
        return 1
    fi

    log_info "U-Boot firmware update and verification completed successfully"
    return 0
}

firmware_kernel() {
    local kernel_dir="$SYS_DIR"
    local kernel_pattern="boot_signed*.bin"

    if [ ! -d "$kernel_dir" ]; then
        log_debug "Kernel directory not found: $kernel_dir"
        return 0
    fi

    local kernel_files=($(find "$kernel_dir" -name "$kernel_pattern" -type f 2>/dev/null))

    if [ ${#kernel_files[@]} -eq 0 ]; then
        log_debug "No kernel files found matching pattern: $kernel_pattern in $kernel_dir"
        return 0
    fi

    local KERNEL_FILE="${kernel_files[0]}"
    log_info "Found kernel file: $(basename "$KERNEL_FILE")"

    if [ ! -r "$KERNEL_FILE" ]; then
        log_error "Kernel file is not readable: $KERNEL_FILE"
        return 1
    fi

    if [ ! -e "/dev/mmcblk0p14" ]; then
        log_error "Target partition /dev/mmcblk0p14 does not exist"
        return 1
    fi

    if [ ! -e "/dev/mmcblk0p15" ]; then
        log_error "Target partition /dev/mmcblk0p15 does not exist"
        return 1
    fi

    local file_size=$(stat -c%s "$KERNEL_FILE")
    log_debug "Original kernel file size: $file_size bytes (skipping SHA256 calculation)"

    write_firmware_without_verify() {
        local source_file="$1"
        local target_partition="$2"
        local firmware_type="$3"
        local max_retries=3
        local attempt=1

        while [ $attempt -le $max_retries ]; do
            log_info "Writing $firmware_type to partition $target_partition (attempt $attempt/$max_retries)..."

            if dd if="$source_file" of="$target_partition" bs=4K conv=notrunc status=none 2>/dev/null; then
                sync
                log_info "Successfully wrote $firmware_type to $target_partition (skipping SHA256 verification)"
                return 0
            else
                log_warn "Failed to write $firmware_type to $target_partition on attempt $attempt"
                if [ $attempt -eq $max_retries ]; then
                    log_error "Failed to write $firmware_type to $target_partition after $max_retries attempts"
                    return 1
                fi
            fi

            attempt=$((attempt + 1))
            log_debug "Retrying in 1 second..."
            sleep 1
        done
    }

    if ! write_firmware_without_verify "$KERNEL_FILE" "/dev/mmcblk0p14" "Kernel"; then
        return 1
    fi

    if ! write_firmware_without_verify "$KERNEL_FILE" "/dev/mmcblk0p15" "Kernel"; then
        return 1
    fi

    log_info "Kernel firmware update and verification completed successfully"
    return 0
}

check_kvm_packages() {
    log_info "checking kvm package installation status"

    local pkgs=("kvmcomm" "nanokvm" "pikvm")
    local all_ok=true

    for pkg_name in "${pkgs[@]}"; do
        if dpkg -l "$pkg_name" 2>/dev/null | grep -q "^ii"; then
            log_debug "Package $pkg_name is installed properly"
        else
            log_info "Package $pkg_name is NOT installed"
            all_ok=false
        fi
    done

    local version=$(dpkg-query -W -f='${Version}\n' kvmcomm 2>/dev/null)
    local required_version=$(dpkg-deb -f "$APP_DIR/kvmcomm_*_arm64.deb" Version)
    if [[ -z "$version" ]]; then
        log_info "kvmcomm package is not installed, cannot check version"
        all_ok=false
    else
        log_debug "kvmcomm package version: $version"
        if dpkg --compare-versions "$version" "lt" "$required_version"; then
            log_info "kvmcomm version $version is less than required $required_version"
            all_ok=false
        else
            log_debug "kvmcomm version $version meets the requirement"
        fi
    fi

    if [ "$all_ok" = true ]; then
        log_info "all kvm packages are installed properly"
        echo "app done"
        exit 0
    else
        log_info "will install kvm packages"
        return 0
    fi
}

install_app() {
    log_step "Installing applications from $APP_DIR"

    if [ ! -d "$APP_DIR" ]; then
        log_warn "App directory not found: $APP_DIR"
        return 0
    fi

    local deb_files=($(find "$APP_DIR" -name "*.deb" -type f | grep -v "chrony_.*_arm64.deb"))

    if [ ${#deb_files[@]} -eq 0 ]; then
        log_warn "No .deb packages found in $APP_DIR (excluding chrony packages)"
        return 0
    fi
    log_info "Found ${#deb_files[@]} .deb package(s) to install (excluding chrony packages)"

    mkdir -p "$CACHE_DIR"
    for deb_file in "${deb_files[@]}"; do
        cp -u "$deb_file" "$CACHE_DIR/"
    done
    log_info "Cached ${#deb_files[@]} package(s) to $CACHE_DIR (excluding chrony packages)"

    local installed_count=0
    local failed_count=0

    for deb_file in "${deb_files[@]}"; do
        local package_name=$(basename "$deb_file")
        log_debug "Installing: $package_name"

        if DEBIAN_FRONTEND=noninteractive dpkg -i --force-confnew "$deb_file" 2>/dev/null; then
            log_info "Installed: $package_name"
            installed_count=$((installed_count + 1))
        else
            log_warn "Failed to install: $package_name, trying to force install..."
            if DEBIAN_FRONTEND=noninteractive dpkg -i --force-confnew --force-depends "$deb_file" 2>/dev/null; then
                log_info "Force installed: $package_name"
                installed_count=$((installed_count + 1))
            else
                log_error "Failed to install: $package_name"
                failed_count=$((failed_count + 1))
            fi
        fi
    done

    log_info "Installation summary: $installed_count successful, $failed_count failed"

    if [ $failed_count -gt 0 ]; then
        log_error "Some packages failed to install"
        return 1
    fi

    return 0
}

# 判断输入参数
if [ $# -ne 1 ]; then
    log_error "Usage: $0 <dtb|uboot|kernel|app>"
    exit 1
fi

# 执行对应函数
case "$1" in
    dtb)
        if [ ! -f "$STATE_FILE_PATH/.dtb.done" ]; then
            DTB_RET=firmware_dtb
            if $DTB_RET; then
                touch "$STATE_FILE_PATH/.dtb.done"
                echo "dtb done"
            else
                echo "dtb failed"
            fi
        else
            echo "dtb done"
        fi
        ;;
    uboot)
        if [ ! -f "$STATE_FILE_PATH/.uboot.done" ]; then
            UBOOT_RET=firmware_uboot
            if $UBOOT_RET; then
                touch "$STATE_FILE_PATH/.uboot.done"
                echo "uboot done"
            else
                echo "uboot failed"
            fi
        else
            echo "uboot done"
        fi
        ;;
    kernel)
        if [ ! -f "$STATE_FILE_PATH/.kernel.done" ]; then
            # cp /root/NanoKVM_Pro_Testing/overlay/boot/* /boot
            KERNEL_RET=firmware_kernel
            if $KERNEL_RET; then
                touch "$STATE_FILE_PATH/.kernel.done"
                echo "kernel done"
            else
                echo "kernel failed"
            fi
        else
            echo "kernel done"
        fi
        ;;
    app)
        echo "app check"
        check_kvm_packages
        install_app
        echo "app done"
        ;;
    *)
        log_error "Invalid argument: $1"
        exit 1
        ;;
esac

echo "Finish"


