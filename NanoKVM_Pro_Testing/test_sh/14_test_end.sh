#!/bin/bash

cp /root/NanoKVM_Pro_Testing/overlay/boot/* /boot/
cp /root/NanoKVM_Pro_Testing/overlay/etc/rc.local /etc/
cp /root/NanoKVM_Pro_Testing/firmware/kvm_spi_ui_test /etc/test-kvm/
cp /root/NanoKVM_Pro_Testing/firmware/kvm_ui_test /etc/test-kvm/
cp /root/NanoKVM_Pro_Testing/firmware/kvm_vin_test /etc/test-kvm/
cp /root/NanoKVM_Pro_Testing/firmware/kvm_ui_setup /usr/bin/kvm_ui_setup
cp /root/NanoKVM_Pro_Testing/config /etc/test-kvm/
rm -f /etc/systemd/system/multi-user.target.wants/ssh.service
rm -f /etc/systemd/system/multi-user.target.wants/usb-gadget.service
rm -f /etc/systemd/system/sockets.target.wants/ssh.socket
systemctl daemon-reload
systemctl enable kvmcomm.service
systemctl start kvmcomm.service
sync
# rm -r /root/*

echo "Finish"
