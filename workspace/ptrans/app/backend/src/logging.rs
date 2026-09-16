//! Инициализация `tracing` для backend.
//!
//! В dev — человекочитаемый формат с цветами.
//! В production — JSON-формат (для ELK/Loki).
//!
//! Уровень задаётся через `RUST_LOG` (по умолчанию `info`).

use tracing_subscriber::{
    fmt, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

/// Инициализирует глобальный tracing subscriber.
///
/// Вызывается **один раз** в `main()` до всего остального.
pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let is_production = std::env::var("APP_ENV")
        .map(|v| v == "production")
        .unwrap_or(false);

    if is_production {
        // JSON-формат для лог-агрегаторов
        tracing_subscriber::registry()
            .with(filter)
            .with(
                fmt::layer()
                    .json()
                    .with_target(true)
                    .with_current_span(true)
                    .with_span_list(true),
            )
            .init();
    } else {
        // Человекочитаемый формат для dev
        tracing_subscriber::registry()
            .with(filter)
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_file(false)
                    .with_line_number(false),
            )
            .init();
    }
}
