use image::{Rgb, RgbImage, ImageBuffer};
use qrcode::QrCode;
use std::ffi::CString;
use rusttype::{Font, Scale, point};
use std::ptr;
  
 // 仅导入必要的特定项，而不是通配符(*)，以避免警告
use winapi::um::wingdi::{
    BITMAPINFO, BITMAPINFOHEADER, BI_RGB, CreateDCA, DeleteDC, EndDoc, EndPage, RGBQUAD,
    StartDocA, StartPage, StretchDIBits, SRCCOPY,
    DIB_RGB_COLORS, DOCINFOA, 
};
use winapi::um::winspool::{EnumPrintersA, PRINTER_ENUM_LOCAL, PRINTER_INFO_1A}; // 添加打印机枚举功能

use std::thread;
use std::time::Duration;

// 打印使能
pub const PRINTER_ENABLE: bool = false;

// 打印机配置常量
pub const TARGET_PRINTER: &str = "CHITENG-CT221B"; // 修改为指定的打印机名称

// USB设备VID/PID配置
const TARGET_VID: u16 = 0x28E9; // 打印机厂商ID（十六进制）
const TARGET_PID: u16 = 0x0290; // 打印机产品ID（十六进制）

const DPI: f32 = 203.0;
const WIDTH_MM: f32 = 50.0;
const HEIGHT_MM: f32 = 20.0;

// 打印机XY轴偏移量（像素）
const PRINTER_X_OFFSET: i32 = 10;
const PRINTER_Y_OFFSET: i32 = 4;

// 日志控制：false=关闭日志，true=开启日志
const LOG_ENABLE: bool = false;

// 自定义日志函数
fn log(msg: &str) {
    if LOG_ENABLE {
        println!("[printer]{}", msg);
    }
}

fn mm_to_pixels(mm: f32) -> i32 {
    ((mm / 25.4) * DPI).round() as i32
}

// 绘制二维码函数
fn draw_qr_code(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>, x: u32, y: u32, size: u32, content: &str) {
    // 计算二维码在打印机上的实际位置，考虑XY轴偏移
    let x = x + PRINTER_X_OFFSET as u32;
    let y = y + PRINTER_Y_OFFSET as u32;
    
    // 创建二维码
    let qr_code = QrCode::new(content).unwrap();
    
    // 计算缩放因子，使二维码适应指定大小
    let modules_per_side = qr_code.width() as u32;
    let scale_factor = size / modules_per_side;
    
    // 绘制二维码
    for py in 0..modules_per_side {
        for px in 0..modules_per_side {
            // 在qrcode 0.13版本中，使用索引访问模块
            let module = qr_code[(px as usize, py as usize)];
            if module == qrcode::types::Color::Dark {
                // 绘制放大的像素块
                for sy in 0..scale_factor {
                    for sx in 0..scale_factor {
                        let pixel_x = x + px * scale_factor + sx;
                        let pixel_y = y + py * scale_factor + sy;
                        if pixel_x < img.width() && pixel_y < img.height() {
                            img.put_pixel(pixel_x, pixel_y, Rgb([0, 0, 0]));
                        }
                    }
                }
            }
        }
    }
}

// 绘制文字函数
fn draw_text(
    img: &mut RgbImage,
    x: u32,          // 整数坐标
    y: u32,          // 整数坐标
    text: &str,
    font_size: u32,  // 整数字体大小
    font_color: [u8; 3],
) {
    // 计算二维码在打印机上的实际位置，考虑XY轴偏移
    let x = x + PRINTER_X_OFFSET as u32;
    let y = y + PRINTER_Y_OFFSET as u32;

    // 1. 使用内置的简单字体数据
    let font_data: &[u8] = include_bytes!("../../fonts/GoogleSans_17pt-Bold.ttf");
    
    let font = match Font::try_from_bytes(font_data) {
        Some(f) => f,
        None => return,
    };
    
    // 2. 设置字体大小（u32 转 f32）
    let scale = Scale::uniform(font_size as f32);
    
    // 3. 计算基线位置并绘制（u32 转 f32）
    let v_metrics = font.v_metrics(scale);
    let start = point(x as f32, y as f32 + v_metrics.ascent);
    
    // 4. 布局和绘制字形
    for glyph in font.layout(text, scale, start) {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|gx, gy, gv| {
                let gx = gx as i32 + bounding_box.min.x;
                let gy = gy as i32 + bounding_box.min.y;
                
                if gx >= 0 
                    && gy >= 0 
                    && (gx as u32) < img.width() 
                    && (gy as u32) < img.height()
                    && gv > 0.3
                {
                    img.put_pixel(gx as u32, gy as u32, Rgb(font_color));
                }
            });
        }
    }
}

// 绘制中文函数
fn draw_chinese(
    img: &mut RgbImage,
    x: u32,          // 整数坐标
    y: u32,          // 整数坐标
    text: &str,
    font_size: u32,  // 整数字体大小
    font_color: [u8; 3],
) {
    // 计算二维码在打印机上的实际位置，考虑XY轴偏移
    let x = x + PRINTER_X_OFFSET as u32;
    let y = y + PRINTER_Y_OFFSET as u32;

    // 1. 使用内置的简单字体数据
    let font_data: &[u8] = include_bytes!("../../fonts/SourceHanSansOLD-Medium-2.otf");
    
    let font = match Font::try_from_bytes(font_data) {
        Some(f) => f,
        None => return,
    };
    
    // 2. 设置字体大小（u32 转 f32）
    let scale = Scale::uniform(font_size as f32);
    
    // 3. 计算基线位置并绘制（u32 转 f32）
    let v_metrics = font.v_metrics(scale);
    let start = point(x as f32, y as f32 + v_metrics.ascent);
    
    // 4. 布局和绘制字形
    for glyph in font.layout(text, scale, start) {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|gx, gy, gv| {
                let gx = gx as i32 + bounding_box.min.x;
                let gy = gy as i32 + bounding_box.min.y;
                
                if gx >= 0 
                    && gy >= 0 
                    && (gx as u32) < img.width() 
                    && (gy as u32) < img.height()
                    && gv > 0.3
                {
                    img.put_pixel(gx as u32, gy as u32, Rgb(font_color));
                }
            });
        }
    }
}

// 绘制线条函数
fn draw_line(img: &mut RgbImage, x1: u32, y1: u32, x2: u32, y2: u32, color: [u8; 3], line_width: u32) {
    // 计算二维码在打印机上的实际位置，考虑XY轴偏移
    let x1 = x1 + PRINTER_X_OFFSET as u32;
    let x2 = x2 + PRINTER_X_OFFSET as u32;
    let y1 = y1 + PRINTER_Y_OFFSET as u32;
    let y2 = y2 + PRINTER_Y_OFFSET as u32;

    let mut x = x1 as i32;
    let mut y = y1 as i32;
    let dx = (x2 as i32 - x1 as i32).abs();
    let dy = (y2 as i32 - y1 as i32).abs();
    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };
    let mut err = dx - dy;
    
    while x != x2 as i32 || y != y2 as i32 {
        // 绘制主线上的像素点
        draw_thick_pixel(img, x as u32, y as u32, color, line_width);
        
        let e2 = err * 2;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
    
    // 确保终点也被绘制（包括线宽）
    draw_thick_pixel(img, x2, y2, color, line_width);
}

// 绘制带线宽的像素点
fn draw_thick_pixel(img: &mut RgbImage, x: u32, y: u32, color: [u8; 3], line_width: u32) {
    // 计算二维码在打印机上的实际位置，考虑XY轴偏移
    let x = x + PRINTER_X_OFFSET as u32;
    let y = y + PRINTER_Y_OFFSET as u32;

    if line_width <= 1 {
        img.put_pixel(x, y, Rgb(color));
        return;
    }
    
    let _half_width = (line_width / 2) as i32;
    let radius = line_width as i32;
    
    // 绘制以(x,y)为中心的圆形区域
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy <= radius * radius {
                let new_x = x as i32 + dx;
                let new_y = y as i32 + dy;
                
                // 检查边界
                if new_x >= 0 && new_y >= 0 && 
                   new_x < img.width() as i32 && new_y < img.height() as i32 {
                    img.put_pixel(new_x as u32, new_y as u32, Rgb(color));
                }
            }
        }
    }
}

fn draw_filled_circle_fast(
    img: &mut RgbImage,
    center_x: u32,
    center_y: u32,
    radius: u32,
    color: [u8; 3],
) {
    // 计算二维码在打印机上的实际位置，考虑XY轴偏移
    let center_x = center_x + PRINTER_X_OFFSET as u32;
    let center_y = center_y + PRINTER_Y_OFFSET as u32;

    if radius == 0 { return; }
    
    let radius_squared = radius * radius;
    
    // 遍历直径范围内的像素
    let start_x = if center_x >= radius { center_x - radius } else { 0 };
    let end_x = (center_x + radius).min(img.width() - 1);
    let start_y = if center_y >= radius { center_y - radius } else { 0 };
    let end_y = (center_y + radius).min(img.height() - 1);

    for y in start_y..=end_y {
        for x in start_x..=end_x {
            let dx = x as i32 - center_x as i32;
            let dy = y as i32 - center_y as i32;
            if (dx * dx + dy * dy) <= radius_squared as i32 {
                img.put_pixel(x, y, Rgb(color));
            }
        }
    }
}

// 获取所有系统打印机的函数
fn get_all_printers() -> Vec<String> {
    unsafe {
        let mut printers: Vec<String> = Vec::new();
        let mut buffer_size: u32 = 0;
        let mut count: u32 = 0;

        // 首先获取所需的缓冲区大小
        EnumPrintersA(
            PRINTER_ENUM_LOCAL,
            ptr::null_mut(),
            1,
            ptr::null_mut(),
            0,
            &mut buffer_size,
            &mut count,
        );

        if buffer_size > 0 {
            let mut buffer = vec![0u8; buffer_size as usize];
            let result = EnumPrintersA(
                PRINTER_ENUM_LOCAL,
                ptr::null_mut(),
                1,
                buffer.as_mut_ptr(),
                buffer_size,
                &mut buffer_size,
                &mut count,
            );

            if result != 0 {
                let mut offset = 0;
                for _ in 0..count {
                    let printer_info = buffer.as_ptr().add(offset) as *const PRINTER_INFO_1A;
                    let name = (*printer_info).pName;
                    
                    if !name.is_null() {
                        let mut name_len = 0;
                        while *name.offset(name_len) != 0 {
                            name_len += 1;
                        }
                        let name_slice = std::slice::from_raw_parts(name as *const u8, name_len as usize);
                        if let Ok(name_str) = String::from_utf8(name_slice.to_vec()) {
                            printers.push(name_str);
                        }
                    }
                    
                    offset += std::mem::size_of::<PRINTER_INFO_1A>();
                }
            }
        }

        printers
    }
}

// 检查指定名称的打印机是否存在（同时检查打印机列表和USB设备）
fn printer_exists(name: &str) -> bool {
    // 首先检查打印机列表
    let printers = get_all_printers();
    let printer_in_list = printers.contains(&name.to_string());
    
    // 如果打印机不在列表中，直接返回不存在
    if !printer_in_list {
        return false;
    }

    log("打印机在列表中");
    
    // 打印机在列表中，但对于特定打印机还需要检查USB设备是否存在
    // 对于名为"CHITENG-CT221B"的打印机，同时检查USB设备是否存在
    if name == TARGET_PRINTER {
        return usb_device_exists(TARGET_VID, TARGET_PID);
    }
    
    // 对于其他打印机，只要在列表中就认为存在
    true
}

// 使用nusb库检查USB设备是否存在
fn usb_device_exists(vid: u16, pid: u16) -> bool {
    // log("查找VID={:04X}, PID={:04X}的USB设备", vid, pid);
    
    match nusb::list_devices() {
        Ok(devices) => {
            // let mut device_count = 0;
            for device in devices {
                // device_count += 1;
                // log("设备 {}: VID={:04X}, PID={:04X}, 制造商='{}', 产品='{}'", 
                //     device_count, 
                //     device.vendor_id(), 
                //     device.product_id(),
                //     device.manufacturer_string().unwrap_or("未知"),
                //     device.product_string().unwrap_or("未知")
                // );
                
                // 检查VID和PID是否匹配
                if device.vendor_id() == vid && device.product_id() == pid {
                    log("✅ 找到匹配的USB设备!");
                    return true;
                }
            }
            
            // log("总共找到 {} 个USB设备", device_count);
            false
        }
        Err(e) => {
            log(&format!("❌ 无法枚举USB设备: {}", e.to_string()));
            // log(&format!("打开串口错误: {:?}", e));
            false
        }
    }
}

// 参数化图像生成函数
pub fn generate_image_with_params(serial: &str, name: &str, exist_wifi: bool) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let width_px = mm_to_pixels(WIDTH_MM) as u32;
    let height_px = mm_to_pixels(HEIGHT_MM) as u32;
    
    // 创建空白图像
    let mut img = ImageBuffer::new(width_px, height_px);
    
    // 填充白色背景
    for pixel in img.pixels_mut() {
        *pixel = Rgb([255, 255, 255]);
    }
    
    // 计算二维码参数
    let qr_width = (width_px as f32 * 0.4) as u32; // 二维码区域占总宽度的40%
    let margin_px = 10u32; // 二维码边距
    let qr_size = qr_width - 2 * margin_px; // 实际二维码大小
    
    // 计算二维码位置（居中偏左）
    let _qr_x = margin_px;
    let _qr_y = (height_px - qr_size) / 2; // 垂直居中
    
    // 绘制二维码（使用传入的串号）
    draw_qr_code(&mut img, 15, 15, 120, serial);
    
    // 绘制二维码下方的文本（使用传入的串号）
    draw_text(&mut img, 15, 127, serial, 25, [0, 0, 0]);

    // 绘制二维码右边的线条
    draw_line(&mut img, 130, 55, 130, 143, [0, 0, 0], 2);
    
    // 绘制右侧的三行文字（使用传入的名称）
    draw_text(&mut img, 137, 10, name, 32, [0, 0, 0]);
    draw_text(&mut img, 155, 47, "Wi-Fi", 50, [0, 0, 0]);
    draw_text(&mut img, 280, 47, ": [   ]", 50, [0, 0, 0]);
    draw_text(&mut img, 155, 102, "POE", 50, [0, 0, 0]);
    draw_text(&mut img, 280, 102, ": [   ]", 50, [0, 0, 0]);

    // 根据WiFi存在与否决定是否绘制实心圆
    if exist_wifi {
        // 绘制Wi-Fi状态的实心圆
        draw_filled_circle_fast(&mut img, 333, 72, 15, [0, 0, 0]);
    }

    // save to output.png
    img.save("../img/output.png").expect("无法保存图像");

    img
}

// 参数化不良贴纸图像生成函数
fn generate_defects_image_with_params(text: &str) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let width_px = mm_to_pixels(WIDTH_MM) as u32;
    let height_px = mm_to_pixels(HEIGHT_MM) as u32;
    let zero_x = 10u32;
    let zero_y = 5u32;
    let line_height = 30u32;
    
    // 创建空白图像
    let mut img = ImageBuffer::new(width_px, height_px);
    
    // 填充白色背景
    for pixel in img.pixels_mut() {
        *pixel = Rgb([255, 255, 255]);
    }

    // 每14个字符或遇到换行符时截取到下一段
    let mut segments = Vec::new();
    let mut current_segment = String::new();
    
    for c in text.chars() {
        if c == '\n' {
            // 遇到换行符，结束当前段
            if !current_segment.is_empty() {
                segments.push(current_segment);
                current_segment = String::new();
            }
        } else {
            current_segment.push(c);
            // 达到14个中文（42个字符），结束当前段
            if current_segment.len() >= 42 {
                segments.push(current_segment);
                current_segment = String::new();
            }
        }
    }
    
    // 添加最后一个未完成的段
    if !current_segment.is_empty() {
        segments.push(current_segment);
    }
    
    // 绘制每一段文本
    for (i, segment) in segments.iter().enumerate() {
        draw_chinese(&mut img, zero_x, zero_y + line_height * i as u32, segment, 24, [0, 0, 0]);
    }

    // save to output.png
    img.save("../img/defects.png").expect("无法保存图像");

    img
}

// 打印图像的优化版本，接受已生成的图像
pub fn print_image(img: &ImageBuffer<Rgb<u8>, Vec<u8>>, printer_name: Option<&str>) -> Result<(), String> {
    unsafe {
        // 1. 获取打印机设备上下文
        let printer_name_cstr = match printer_name {
            Some(name) => {
                let cstr = CString::new(name).map_err(|e| e.to_string())?;
                Some(cstr)
            }
            None => None,
        };

        let printer_name_ptr = printer_name_cstr
            .as_ref()
            .map(|cstr| cstr.as_ptr())
            .unwrap_or(ptr::null());

        // 准备打印文档信息
        let doc_name = CString::new("Rust Print Demo").map_err(|e| e.to_string())?;
        let doc_info = DOCINFOA {
            cbSize: std::mem::size_of::<DOCINFOA>() as i32,
            lpszDocName: doc_name.as_ptr(),
            lpszOutput: ptr::null(),
            lpszDatatype: ptr::null(),
            fwType: 0,
        };

        let hdc = CreateDCA(
            b"WINSPOOL\0".as_ptr() as *const i8,
            printer_name_ptr,
            ptr::null(),
            ptr::null(),
        );

        if hdc.is_null() {
            return Err("无法创建打印机设备上下文".to_string());
        }

        // 2. 开始打印作业
        if StartDocA(hdc, &doc_info) <= 0 {
            DeleteDC(hdc);
            return Err("无法开始打印作业".to_string());
        }

        if StartPage(hdc) <= 0 {
            EndDoc(hdc);
            DeleteDC(hdc);
            return Err("无法开始新页面".to_string());
        }

        // 3. 使用传入的图像并转换为位图
        let width = img.width() as i32;
        let height = img.height() as i32;

        // 创建BITMAPINFO结构
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // 负值表示从上到下的位图
                biPlanes: 1,
                biBitCount: 24, // 24位色: R, G, B 各8位
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: (DPI as f32 * 39.3701) as i32, // 设置DPI
                biYPelsPerMeter: (DPI as f32 * 39.3701) as i32,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD { rgbBlue: 0, rgbGreen: 0, rgbRed: 0, rgbReserved: 0 }],
        };

        // 4. 将图像数据复制到临时缓冲区（24位RGB格式）
        let mut image_data = Vec::<u8>::new();
        for pixel in img.pixels() {
            image_data.push(pixel.0[2]); // B
            image_data.push(pixel.0[1]); // G
            image_data.push(pixel.0[0]); // R
        }

        // 5. 打印图像
        let result = StretchDIBits(
            hdc,
            0, 0, // 目标位置
            width, height, // 目标大小
            0, 0, // 源位置
            width, height, // 源大小
            image_data.as_ptr() as *const winapi::ctypes::c_void,
            &bmi,
            DIB_RGB_COLORS,
            SRCCOPY,
        );

        // 6. 结束打印
        if result as u32 == 0xFFFFFFFFu32 {
            EndPage(hdc);
            EndDoc(hdc);
            DeleteDC(hdc);
            return Err("图像打印失败".to_string());
        }

        if EndPage(hdc) <= 0 {
            EndDoc(hdc);
            DeleteDC(hdc);
            return Err("无法结束页面".to_string());
        }

        if EndDoc(hdc) <= 0 {
            DeleteDC(hdc);
            return Err("无法结束文档".to_string());
        }

        DeleteDC(hdc);
        Ok(())
    }
}

pub async fn is_printer_connected() -> bool {
    printer_exists(TARGET_PRINTER)
}

pub fn spawn_printer_task() {
    thread::spawn(move || {
        // 测试打印功能
        log("\n=== 打印测试 ===");
    
        let test_serial = "Neal00150";
        let test_name = "NanoKVM-ATX-B";

        let _img0 = generate_defects_image_with_params("不良贴纸不良\n贴纸不良贴纸不良贴纸不良贴纸贴纸不良贴纸不良贴纸贴纸不良贴纸不良贴纸"); // 测试换行符分割
        
        if printer_exists(TARGET_PRINTER) {
            let img = generate_image_with_params(test_serial, test_name, true);
            
            if PRINTER_ENABLE {
                match print_image(&img, Some(TARGET_PRINTER)) {
                    Ok(_) => log("✅ 打印测试成功"),
                    Err(e) => log(&format!("❌ 打印测试失败: {}", e.to_string())),
                }
            } else {
                log(&format!("打印功能已禁用"));
            }
        } else {
            log(&format!("❌ 设备不存在: {}", test_name));
        }
        
        loop {
            thread::sleep(Duration::from_secs(1));
        }
    });
}