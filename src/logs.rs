use chrono::Local;
use tracing::Subscriber;
use tracing_subscriber::{
    fmt::{self, format::Writer, FmtContext, FormatEvent, FormatFields},
    layer::SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
    EnvFilter,
};

const ERROR_COLORED: &str = "\x1b[31mERROR\x1b[0m";
const WARN_COLORED: &str = "\x1b[33mWARN \x1b[0m";
const INFO_COLORED: &str = "\x1b[32mINFO \x1b[0m";
const DEBUG_COLORED: &str = "\x1b[34mDEBUG\x1b[0m";
const TRACE_COLORED: &str = "\x1b[35mTRACE\x1b[0m";

pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug".into())
        )
        .with(fmt::layer().event_format(CustomEventFormatter))
        .init();
}

// 自定义事件格式化器，缩短模块路径
struct CustomEventFormatter;

impl<S, N> FormatEvent<S, N> for CustomEventFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        let meta = event.metadata();

        // 时间戳
        let now = Local::now();
        write!(writer, "{} ", now.format("%Y-%m-%d %H:%M:%S"))?;

        #[cfg(debug_assertions)]
        {
            // 日志级别（带颜色）
            let level = meta.level();
            let level_str = match *level {
                tracing::Level::ERROR => ERROR_COLORED,
                tracing::Level::WARN => WARN_COLORED,
                tracing::Level::INFO => INFO_COLORED,
                tracing::Level::DEBUG => DEBUG_COLORED,
                tracing::Level::TRACE => TRACE_COLORED,
            };
            write!(writer, "{} ", level_str)?;
        }

        #[cfg(not(debug_assertions))]
        {
            // 日志级别
            write!(writer, "{:<5} ", meta.level())?;
        }

        // 缩短后的模块路径（保留最后两级）
        // let target = meta.target();
        // let parts: Vec<&str> = target.split("::").collect();
        // let short_target = if parts.len() > 2 {
        //     parts[parts.len() - 2..].join("::")
        // } else {
        //     target.to_string()
        // };
        // write!(writer, "{}: ", short_target)?;

        // 文件名和行号
        if let Some(file) = meta.file() && let Some(line) = meta.line() {
            write!(writer, "{}:{}: ", file, line)?;
        }

        // 事件字段内容
        ctx.field_format().format_fields(writer.by_ref(), event)?;

        writeln!(writer)
    }
}
