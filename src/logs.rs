use std::io::IsTerminal;

use time::macros::format_description;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt::time::LocalTime, prelude::*};

pub fn init_tracing() {
    // 判断是否支持彩色输出
    let enable_color = std::io::stdout().is_terminal();
    // 设置日志格式
    let fmt_timer = LocalTime::new(format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"));
    
    // 控制台输出层
    let console_layer = tracing_subscriber::fmt::layer()
        // 设置时间格式
        .with_timer(fmt_timer.clone())
        // 是否启用彩色输出
        .with_ansi(enable_color)
        // 显示模块路径
        .with_target(true)
        // 显示文件名
        .with_file(true)
        // 显示行号
        .with_line_number(true);

    // 创建日志目录
    let log_dir = "logs";
    std::fs::create_dir_all(log_dir).ok();

    // 配置日志轮转：每天轮转一次
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY) // 每天轮转
        .filename_prefix("app") // 文件名前缀
        .filename_suffix("log") // 文件名后缀
        .max_log_files(7) // 保留最多7个日志文件（7天）
        .build(log_dir)
        .expect("初始化日志文件失败");

    // 文件输出层
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_appender)
        .with_timer(fmt_timer)
        .with_ansi(false) // 文件中不使用ANSI颜色
        .with_target(true)
        .with_line_number(true);

    let registry = tracing_subscriber::registry()
        // 从环境变量读取日志级别配置
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        // 记录错误信息
        .with(ErrorLayer::default());

    // 如果是终端环境，同时输出到控制台；否则只输出到文件
    if cfg!(debug_assertions) {
        registry.with(console_layer).init();
    } else {
        registry.with(file_layer).init();
    }
}
