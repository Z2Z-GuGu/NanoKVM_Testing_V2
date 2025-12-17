# 如何检测NanoKVM启动的状态

## 应该有的几种状态：
  + 未连接，等待连接
    + 特征：连续5s没有打印任何字符，发送回车无响应
  + 启动中，等待启动完成
    + 特征：一直在打印："[  OK  ]"，间隔不超过5s
    + 实际的OK带有颜色标识对应16进制如下
      + 5B 1B 5B 30 3B 33 32 6D 20 20 4F 4B 20 20 1B 5B 30 6D 5D
      + [  ESC[  0  ;  3  2  m        O  K        ESC[  0  m  ]
  + 启动成功，等待输入账号密码
    + 特征：捕捉到字符："login:"
  + 进入系统，等待输入命令
    + 特征：等待出现字符："Welcome";->;等待出现字符："#"
  + 无网络连接
    + 执行"ifconfig eth0 | grep 192"
    + 回复: 69 6E 65 74 20 1B 5B 30 31 3B 33 31 6D 1B 5B 4B 31 39 32 1B 5B 6D 1B 5B 4B 2E 31 36 38 2E 31 2E 31 30 39 20
    + 翻译: i  n  e  t  空 ESC[  0  1  ;  3  1  m  ESC[  K  1  9  2  ESC[  m  ESC [ K  .  1  6  8  .  1  .  1  0  9  空格
    + 如果无任何有效信息则重新执行
      + grep: eth0: No such file or directory
      + 仅换行
  + 启动失败（待补充）
  + 已经正常安装
  + 未正常安装（可能是经过n次产测，产测不通过）

## 正常的启动日志如下：
```shell
+ 插入瞬间COM会消失（不确定是什么原因，可能是电源，也可能是串口自己掉的，认为不能作为判断依据）
[    2.451664] systemd[1]: Queued start job for default target Graphical Interface.
[    2.460441] random: systemd: uninitialized urandom read (16 bytes read)
[    2.468273] systemd[1]: Created slice Slice /system/getty.
[  OK  ] Created slice Slice /system/getty.
[    2.505417] random: systemd: uninitialized urandom read (16 bytes read)
[    2.512910] systemd[1]: Created slice Slice /system/modprobe.
[  OK  ] Created slice Slice /system/modprobe.
[    2.546250] systemd[1]: Created slice Slice /system/serial-getty.
[  OK  ] Created slice Slice /system/serial-getty.
[    2.585847] systemd[1]: Created slice User and Session Slice.
[  OK  ] Created slice User and Session Slice.
[    2.615721] systemd[1]: Started Dispatch Password Requests to Console Directory Watch.
[  OK  ] Started Dispatch Password …ts to Console Directory Watch.
[    2.655684] systemd[1]: Started Forward Password Requests to Wall Directory Watch.
[  OK  ] Started Forward Password R…uests to Wall Directory Watch.
[    2.695587] systemd[1]: Condition check resulted in Arbitrary Executable File Formats File System Automount Point being skipped.
[    2.707415] systemd[1]: Reached target Local Encrypted Volumes.
[  OK  ] Reached target Local Encrypted Volumes.
[    2.735613] systemd[1]: Reached target Remote File Systems.
[  OK  ] Reached target Remote File Systems.
[    2.765403] systemd[1]: Reached target Slice Units.
[  OK  ] Reached target Slice Units.
[    2.795449] systemd[1]: Reached target Swaps.
[  OK  ] Reached target Swaps.
[    2.835421] systemd[1]: Reached target System Time Set.
[  OK  ] Reached target System Time Set.
[    2.875557] systemd[1]: Reached target Local Verity Protected Volumes.
[  OK  ] Reached target Local Verity Protected Volumes.
[    2.905898] systemd[1]: Listening on Syslog Socket.
[  OK  ] Listening on Syslog Socket.
[    2.935864] systemd[1]: Listening on fsck to fsckd communication Socket.
[  OK  ] Listening on fsck to fsckd communication Socket.
[    2.975663] systemd[1]: Listening on initctl Compatibility Named Pipe.
[  OK  ] Listening on initctl Compatibility Named Pipe.
[    3.029510] systemd[1]: Condition check resulted in Journal Audit Socket being skipped.
[    3.038214] systemd[1]: Listening on Journal Socket (/dev/log).
[  OK  ] Listening on Journal Socket (/dev/log).
[    3.075993] systemd[1]: Listening on Journal Socket.
[  OK  ] Listening on Journal Socket.
[    3.106086] systemd[1]: Listening on udev Control Socket.
[  OK  ] Listening on udev Control Socket.
[    3.135860] systemd[1]: Listening on udev Kernel Socket.
[  OK  ] Listening on udev Kernel Socket.
[    3.167468] systemd[1]: Mounting Huge Pages File System...
         Mounting Huge Pages File System...
[    3.195798] systemd[1]: Condition check resulted in POSIX Message Queue File System being skipped.
[    3.207027] systemd[1]: Mounting Kernel Debug File System...
         Mounting Kernel Debug File System...
[    3.235825] systemd[1]: Condition check resulted in Kernel Trace File System being skipped.
[    3.244918] systemd[1]: systemd-journald.service: unit configures an IP firewall, but the local system does not support BPF/cgroup firewalling.
[    3.257882] systemd[1]: (This warning is only shown for the first unit using IP firewalling.)
[    3.268177] systemd[1]: Starting Journal Service...
         Starting Journal Service...
[    3.305941] systemd[1]: Condition check resulted in Create List of Static Device Nodes being skipped.
[    3.318384] systemd[1]: Starting Load Kernel Module configfs...
         Starting Load Kernel Module configfs...
[    3.358043] systemd[1]: Starting Load Kernel Module drm...
         Starting Load Kernel Module drm...
[    3.388497] systemd[1]: Starting Load Kernel Module efi_pstore...
         Starting Load Kernel Module efi_pstore...
[    3.428093] systemd[1]: Starting Load Kernel Module fuse...
         Starting Load Kernel Module fuse...
[    3.457869] systemd[1]: Started Nameserver information manager.
[  OK  ] Started Nameserver information manager.
[    3.500004] systemd[1]: Reached target Preparation for Network.
[  OK  ] Reached target Preparation for Network.
[    3.535740] systemd[1]: Condition check resulted in File System Check on Root Device being skipped.
[    3.548625] systemd[1]: Starting Load Kernel Modules...
         Starting Load Kernel Modules...
[    3.577797] systemd[1]: Starting Remount Root and Kernel File Systems...
         Starting Remount Root and Kernel File Systems...
[    3.599780] EXT4-fs (mmcblk0p17): re-mounted. Opts: errors=remount-ro
[    3.617993] systemd[1]: Starting Coldplug All udev Devices...
         Starting Coldplug All udev Devices...
[    3.649863] systemd[1]: Started Journal Service.
[  OK  ] Started Journal Service.
[  OK  ] Mounted Huge Pages File System.
[  OK  ] Mounted Kernel Debug File System.
[  OK  ] Finished Load Kernel Module configfs.
[  OK  ] Finished Load Kernel Module drm.
[  OK  ] Finished Load Kernel Module efi_pstore.
[  OK  ] Finished Load Kernel Module fuse.
[  OK  ] Finished Load Kernel Modules.
[  OK  ] Finished Remount Root and Kernel File Systems.
         Mounting FUSE Control File System...
         Mounting Kernel Configuration File System...
         Starting Flush Journal to Persistent Storage...
[    4.028531] systemd-journald[1125]: Received client request to flush runtime journal.
         Starting Load/Save Random Seed...
[    4.046159] systemd-journald[1125]: File /var/log/journal/770d8359a0c2447ea35ff19e4ae4d7d9/system.journal corrupted or uncleanly shut down, renaming and replacing.
         Starting Apply Kernel Variables...
         Starting Create System Users...
[  OK  ] Finished Coldplug All udev Devices.
[  OK  ] Mounted FUSE Control File System.
[  OK  ] Mounted Kernel Configuration File System.
[  OK  ] Finished Flush Journal to Persistent Storage.
[  OK  ] Finished Apply Kernel Variables.
         Starting Helper to synchronize boot up for ifupdown...
[  OK  ] Finished Helper to synchronize boot up for ifupdown.
[  OK  ] Finished Create System Users.
         Starting Create Static Device Nodes in /dev...
[  OK  ] Finished Create Static Device Nodes in /dev.
[  OK  ] Reached target Preparation for Local File Systems.
         Mounting /tmp...
         Starting Rule-based Manage…for Device Events and Files...
[  OK  ] Mounted /tmp.
[  OK  ] Started Rule-based Manager for Device Events and Files.
[  OK  ] Created slice Slice /system/systemd-backlight.
         Starting Load/Save Screen …ness of backlight:backlight...
[  OK  ] Finished Load/Save Screen …htness of backlight:backlight.
[  OK  ] Found device /dev/mmcblk0p16.
[  OK  ] Found device /dev/ttyS0.
[  OK  ] Reached target Hardware activated USB gadget.
         Mounting /boot...
         Starting Load Kernel Module efi_pstore...
[  OK  ] Mounted /boot.
[  OK  ] Finished Load Kernel Module efi_pstore.
[  OK  ] Reached target Local File Systems.
[  OK  ] Started ifup for eth0.
         Starting Raise network interfaces...
         Starting Create Volatile Files and Directories...
         Starting Run usb gadget...
[  OK  ] Finished Create Volatile Files and Directories.
         Starting Network Name Resolution...
         Starting Record System Boot/Shutdown in UTMP...
[  OK  ] Finished Record System Boot/Shutdown in UTMP.
[  OK  ] Reached target System Initialization.
[  OK  ] Started resolvconf-pull-resolved.path.
[  OK  ] Started Daily Cleanup of Temporary Directories.
[  OK  ] Reached target Path Units.
[  OK  ] Listening on Avahi mDNS/DNS-SD Stack Activation Socket.
[  OK  ] Reached target Socket Units.
[  OK  ] Reached target Basic System.
[  OK  ] Listening on D-Bus System Message Bus Socket.
         Starting Save/Restore Sound Card State...
         Starting Avahi mDNS/DNS-SD Stack...
[  OK  ] Started Regular background program processing daemon.
[  OK  ] Started D-Bus System Message Bus.
[  OK  ] Started Save initial kernel messages after boot.
         Starting Remove Stale Onli…t4 Metadata Check Snapshots...
         Starting Dispatcher daemon for systemd-networkd...
         Starting System Logging Service...
[  OK  ] Started System Device Services.
         Starting User Login Management...
[  OK  ] Started TEE Supplicant.
         Starting LSB: Start busybox udhcpd at boot time...
         Starting WPA supplicant...
[  OK  ] Started Network Name Resolution.
[  OK  ] Finished Raise network interfaces.
[  OK  ] Started Run usb gadget.
[  OK  ] Finished Save/Restore Sound Card State.
[  OK  ] Started System Logging Service.
[  OK  ] Started Avahi mDNS/DNS-SD Stack.
[  OK  ] Started LSB: Start busybox udhcpd at boot time.
[  OK  ] Started WPA supplicant.
[  OK  ] Reached target Network.
[  OK  ] Reached target Network is Online.
[  OK  ] Reached target Host and Network Name Lookups.
[  OK  ] Reached target Sound Card.
         Starting chrony, an NTP client/server...
         Starting LSB: Brings up/down network automatically...
         Starting A high performanc… and a reverse proxy server...
         Starting /etc/rc.local Compatibility...
         Starting resolvconf-pull-resolved.service...
[    7.956189] rc.local[1990]: /etc/rc.local: line 5: bash/etc/init.d/axemac.sh: No such file or directory
         Starting OpenBSD Secure Shell server...
         Starting Permit User Sessions...
[    8.002332] rc.local[1998]: run auto_load_all_drv.sh start
[    8.110249] [HYN][enter]hyn_ts_init
[    8.114247] [HYN][enter]hyn_ts_probe
[  OK  ] Started ttyd daemon.
[    8.131442] [HYN][enter]hyn_parse_dt
[    8.140589] [HYN][enter]hyn_power_source_ctrl
[    8.150742] [HYN][enter]cst8xxT_init
[  OK  ] Started Run wifi.sh.
[  OK  ] Finished Load/Save Random Seed.
[  OK  ] Finished Permit User Sessions.
[  OK  ] Started User Login Management.
[  OK  ] Started A high performance…er and a reverse proxy server.
[  OK  ] Finished resolvconf-pull-resolved.service.
[  OK  ] Started OpenBSD Secure Shell server.
[  OK  ] Started LSB: Brings up/down network automatically.
[  OK  ] [    8.660576] [HYN][enter]hyn_input_dev_init
Started chrony, an NTP client/server.
[  OK  ] Reached target System Time Synchronized.
[    8.705765] [HYN][enter]hyn_proximity_int
[  OK  ] Started Daily apt download activities.
[  OK  ] Started Daily apt upgrade and clean activities.
[  OK  ] Started Daily dpkg database backup timer.
[  OK  ] Started Periodic ext4 Onli…ata Check for All Filesystems.
[    8.741708] rc.local[1998]: insmod ax_cmm, param: cmmpool=anonymous,0,0x6c000000,320M
[  OK  ] Started Discard unused blocks once a week.
[  OK  ] Started Daily rotation of log files.
[  OK  ] Started Message of the Day.
[  OK  ] Reached target Timer Units.
[  OK  ] Started ISC DHCP IPv4 server.
[  OK  ] Started ISC DHCP IPv6 server.
[  OK  ] Finished Remove Stale Onli…ext4 Metadata Check Snapshots.
[    9.622621] ieee80211 phy0:
[    9.622621] *******************************************************
[    9.622621] ** CAUTION: USING PERMISSIVE CUSTOM REGULATORY RULES **
[    9.622621] *******************************************************
[  OK  ] Started Dispatcher daemon for systemd-networkd.
         Starting Bluetooth service...
[  OK  ] Started Bluetooth service.
[  OK  ] Reached target Bluetooth Support.
[   10.138167] rc.local[1998]: run auto_load_all_drv.sh end
         Starting Hostname Service...
[   10.200936] rc.local[2489]: run npu_set_bw_limiter.sh start
[   10.260524] rc.local[2489]: already register bw limit for NPU
[   10.290592] rc.local[2489]: this chip type is AX630C_CHIP
[  OK  ] Started /etc/rc.local Compatibility.
[   10.320467] rc.local[2489]: run npu_set_bw_limiter.sh end
[  OK  ] Started Getty on tty1.
[   10.381810] rc.local[2507]: Starting check system
[  OK  ] Started Serial Getty on ttyS0.
[   10.440517] rc.local[2507]: set slota bootable is true
[   10.501684] rc.local[2514]: Starting ota check
[  OK  ] Reached target Login Prompts.
[  OK  ] Reached target Multi-User System.
[  OK  ] Reached target Graphical Interface.
         Starting Record Runlevel Change in UTMP...
[  OK  ] Started Hostname Service.
[  OK  ] Finished Record Runlevel Change in UTMP.

Ubuntu 22.04.5 LTS kvm-0733 ttyS0

kvm-0733 login: root
Password:
Welcome to Ubuntu 22.04.5 LTS (GNU/Linux 4.19.125 aarch64)

 * Documentation:  https://help.ubuntu.com
 * Management:     https://landscape.canonical.com
 * Support:        https://ubuntu.com/pro

This system has been minimized by removing packages and content that are
not required on a system that users do not log into.

To restore this content, you can run the 'unminimize' command.

The programs included with the Ubuntu system are free software;
the exact distribution terms for each program are described in the
individual files in /usr/share/doc/*/copyright.

Ubuntu comes with ABSOLUTELY NO WARRANTY, to the extent permitted by
applicable law.

root@kvm-0733:~# ifconfig
eth0: flags=4099<UP,BROADCAST,MULTICAST>  mtu 1500
        ether 48:da:35:6d:07:33  txqueuelen 1000  (Ethernet)
        RX packets 0  bytes 0 (0.0 B)
        RX errors 0  dropped 0  overruns 0  frame 0
        TX packets 0  bytes 0 (0.0 B)
        TX errors 0  dropped 0 overruns 0  carrier 0  collisions 0
        device interrupt 28

lo: flags=73<UP,LOOPBACK,RUNNING>  mtu 65536
        inet 127.0.0.1  netmask 255.0.0.0
        inet6 ::1  prefixlen 128  scopeid 0x10<host>
        loop  txqueuelen 1000  (Local Loopback)
        RX packets 86  bytes 7141 (7.1 KB)
        RX errors 0  dropped 0  overruns 0  frame 0
        TX packets 86  bytes 7141 (7.1 KB)
        TX errors 0  dropped 0 overruns 0  carrier 0  collisions 0

usb0: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500
        inet 10.202.108.1  netmask 255.255.255.0  broadcast 0.0.0.0
        inet6 fe80::4ada:35ff:fe6d:ca6f  prefixlen 64  scopeid 0x20<link>
        ether 48:da:35:6d:ca:6f  txqueuelen 1000  (Ethernet)
        RX packets 0  bytes 0 (0.0 B)
        RX errors 0  dropped 0  overruns 0  frame 0
        TX packets 0  bytes 0 (0.0 B)
        TX errors 0  dropped 0 overruns 0  carrier 0  collisions 0

usb1: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500
        inet 10.202.109.1  netmask 255.255.255.0  broadcast 0.0.0.0
        inet6 fe80::4ada:35ff:fe6d:ca6e  prefixlen 64  scopeid 0x20<link>
        ether 48:da:35:6d:ca:6e  txqueuelen 1000  (Ethernet)
        RX packets 163  bytes 25308 (25.3 KB)
        RX errors 0  dropped 4  overruns 0  frame 0
        TX packets 25  bytes 6764 (6.7 KB)
        TX errors 0  dropped 0 overruns 0  carrier 0  collisions 0

root@kvm-0733:~#


```

## 如何通过串口管理与检测状态