use std::ffi::CStr;
use std::ffi::CString;

use tracing::level_filters::LevelFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;
use anyhow::anyhow;

pub fn init() -> anyhow::Result<()> {
    let env = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .from_env()?;

    let stdout = tracing_subscriber::fmt::layer()
                .without_time()
                .with_target(false)
                .with_thread_names(true);

    let syslog = {
        static IDENTITY: &'static CStr = c"notificationd";
        let (options, facility) = Default::default();
        let writer = syslog_tracing::Syslog::new(IDENTITY, options, facility)
            .ok_or(anyhow!("failed to create syslog writer"))?;
        tracing_subscriber::fmt::layer()
                .without_time()
                .with_target(false)
                .with_thread_names(true)
                .with_ansi(false)
                .with_writer(writer)
    };

    println!("{env:?}");
    let registry = tracing_subscriber::registry()
        .with(env)
        .with(stdout)
        .with(syslog);
    tracing::subscriber::set_global_default(registry)?;
    Ok(())
}

/// Trait for logging different kinds of errors
pub trait LogError {
    /// If this result is an error, log it as such
    fn log(self) -> Self;
}

impl<T> LogError for std::io::Result<T> {
    fn log(self) -> Self {
        if let Err(err) = &self {
            tracing::error!("{err:?}");
        }
        self
    }
}

impl<T> LogError for anyhow::Result<T> {
    fn log(self) -> Self {
        if let Err(err) = &self {
            tracing::error!("{err:?}");
        }
        self
    }
    
}
