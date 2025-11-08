use std::io::IsTerminal;

use time::macros::format_description;
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt::time::LocalTime, prelude::*};

pub fn init_tracing() {
    // 判断是否支持彩色输出
    let enable_color = std::io::stdout().is_terminal();
    // 设置日志格式
    let fmt_timer = LocalTime::new(format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"));
    let fmt_layer = tracing_subscriber::fmt::layer()
        // 设置时间格式
        .with_timer(fmt_timer)
        // 是否启用彩色输出
        .with_ansi(enable_color)
        // 显示模块路径
        .with_target(true)
        // 显示行号
        .with_line_number(true);
    
    // 调试模式下显示文件名
    #[cfg(debug_assertions)]
    let fmt_layer = fmt_layer.with_file(true);

    tracing_subscriber::registry()
        // 从环境变量读取日志级别配置
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        // 记录错误信息
        .with(ErrorLayer::default())
        // 加载日志格式
        .with(fmt_layer)
        .init();
}
