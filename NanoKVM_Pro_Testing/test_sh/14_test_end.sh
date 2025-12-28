#!/bin/bash

cp /root/NanoKVM_Pro_Testing/overlay/boot/* /boot/
cp /root/NanoKVM_Pro_Testing/overlay/etc/rc.local /etc/
cp /root/NanoKVM_Pro_Testing/firmware/kvm_spi_ui_test /etc/test-kvm/
cp /root/NanoKVM_Pro_Testing/firmware/kvm_ui_test /etc/test-kvm/
cp /root/NanoKVM_Pro_Testing/config /etc/test-kvm/
systemctl enable kvmcomm.service
systemctl start kvmcomm.service
sync
# rm -r /root/*

echo "Finish"

