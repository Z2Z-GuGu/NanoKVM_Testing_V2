# 串口终端功能项目需求

这是一个tuari项目，前端使用React+Typescript做框架，后端使用rust，使用pnpm tauri dev做测试，只需要考虑tauri下的效果，无需做网页兼容，所以测试由我来做。
实现效果图如下：
![串口终端明亮模式效果图](img/Light-UartTerminal.png)
![串口终端暗黑模式效果图](img/Dark-UartTerminal.png)
在现有框架下增加串口终端功能，后端负责串口设备的自动扫描和连接，前端负责串口数据的显示和发送。以下为实现细则：

## 后端
+ 在rust后端新增串口功能的线程，自动扫描并连接可用的串口设备。
    + 串口设备的扫描要多平台兼容
        + Linux下连接ttyUSB、ttyACM开头的设备
        + Windows下连接COM开头的设备
        + macOS下连接cu.开头的设备
+ 串口自动连接的设备要筛选其USB VID和PID，只连接USB VID为0x1a86，PID为0x7523的设备
+ 串口波特率为1500000，数据位8位，无校验位，1位停止位
+ 串口连接成功后，后端与前端通过WebSocket建立连接，前端发送的数据通过WebSocket发送给后端，后端收到数据后通过串口发送给设备。
+ 串口连接失败或断开后，后端要自动尝试重新连接
+ 串口的连接和断开信息也要通过WebSocket发送给前端
+ 代码写到src-tauri/src/threads/serial.rs，要求简洁易懂

## 前端
+ 当前前端已经有一个用于显示串口数据的组件，位于src/components/Terminal.tsx，在此基础上做显示功能，注意字体大小，行间距，显示位置不做更改
+ 当前前端可能存在一些错误的通信机制，请删除，统一使用WebSocket与后端通信
+ 借助xterm.js库，格式化串口的收发信息，显示来自后端WebSocket的消息，同时支持发送数据到串口设备
+ 当接收到后端发送的断开/连接消息时，在Terminal中显示"[disconnected]\n"或"[connected]\n"
+ 不要添加额外的功能，维持串口组件的清爽