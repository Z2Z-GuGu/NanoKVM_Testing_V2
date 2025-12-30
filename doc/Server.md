上传测试项目结果

- URL: https://maixvision.sipeed.com/api/v1/nanokvm/test-items
- 类型：POST
- 请求头：
  - token: MaixVision2024
参数：

| 参数 | 类型 | 说明 |
| ---- | ---- | ---- |
| uid | string | 必选，设备 UID |
| serial | string | 必选，设备序列号 |
| hardware | string | 必选，设备硬件型号 |
| app | string | 可选，app 测试结果 |
| atx | string | 可选，atx 测试结果 |
| emmc | string | 可选，emmc 测试结果 |
| eth | string | 可选，eth 测试结果 |
| lt6911 | string | 可选，lt6911 测试结果 |
| lt86102 | string | 可选，lt86102 测试结果 |
| rotary | string | 可选，rotary 测试结果 |
| screen | string | 可选，screen 测试结果 |
| sdcard | string | 可选，sdcard 测试结果 |
| touch | string | 可选，touch 测试结果 |
| uart | string | 可选，uart 测试结果 |
| usb | string | 可选，usb 测试结果 |
| wifi | string | 可选，wifi 测试结果 |
| ws2812 | string | 可选，ws2812 测试结果 |

返回

| 参数 | 类型 | 说明 |
| ---- | ---- | ---- |
| code | int | 返回码，0-成功；其它-失败 |
| msg | string | 返回信息 |

查询测试项目列表

- URL: https://maixvision.sipeed.com/api/v1/nanokvm/test-items
- 类型：GET
- 请求头：
  - token: MaixVision2024
参数

| 参数 | 类型 | 说明 |
| ---- | ---- | ---- |
| serial | string | 可选，设备序列号 |
| uid | string | 可选，设备 UID |

serial 和 uid 不能同时为空，必须提供一个。
如果同时提供，则以 serial 为准。

返回

| 参数 | 类型 | 说明 |
| ---- | ---- | ---- |
| code | int | 返回码，0-成功；其它-失败 |
| msg | string | 返回信息 |
| list | []TestItem | 测试项目列表 |

Item:

| 参数 | 类型 | 说明 |
| ---- | ---- | ---- |
| uid | string | 设备 UID |
| serial | string | 设备序列号 |
| hardware | string | 设备硬件型号 |
| app | string | app 测试结果 |
| atx | string | atx 测试结果 |
| emmc | string | emmc 测试结果 |
| eth | string | eth 测试结果 |
| lt6911 | string | lt6911 测试结果 |
| lt86102 | string | lt86102 测试结果 |
| rotary | string | rotary 测试结果 |
| screen | string | screen 测试结果 |
| sdcard | string | sdcard 测试结果 |
| touch | string | touch 测试结果 |
| uart | string | uart 测试结果 |
| usb | string | usb 测试结果 |
| wifi | string | wifi 测试结果 |
| ws2812 | string | ws2812 测试结果 |
| updated | string | 更新时间 |
| created | string | 创建时间 |


上传测试通过数据

- URL: https://maixvision.sipeed.com/api/v1/nanokvm/test-result
- 类型：POST
- 请求头：
  - token: MaixVision2024
  - passwd: Sipeed.NanoKVM@25
参数

| 参数 | 类型 | 说明 |
| ---- | ---- | ---- |
| serial | string | 必选，设备序列号 |
| uid | string | 可选，设备 UID |
| status | string | 必选，测试状态，pass-通过，其他-失败 |

返回

| 参数 | 类型 | 说明 |
| ---- | ---- | ---- |
| code | int | 返回码，0-成功；其它-失败 |
| msg | string | 返回信息 |

查询测试通过数据

- URL: https://maixvision.sipeed.com/api/v1/nanokvm/test-result
- 类型：GET
- 请求头：
  - token: MaixVision2024
参数

| 参数 | 类型 | 说明 |
| ---- | ---- | ---- |
| serial | string | 可选，设备序列号 |
| uid | string | 可选，设备 UID |

serial 和 uid 不能同时为空，必须提供一个。
如果同时提供，则以 serial 为准。

返回

| 参数 | 类型 | 说明 |
| ---- | ---- | ---- |
| code | int | 返回码，0-成功；其它-失败 |
| msg | string | 返回信息 |
| device | Device | 设备信息 |

Device:

| 参数 | 类型 | 说明 |
| ---- | ---- | ---- |
| uid | string | 设备 UID |
| serial | string | 设备序列号 |
| created | string | 创建时间 |
