use thiserror::Error;
use crate::system::SystemId;

#[derive(Error, Debug)]
pub enum TX2Error {
    #[error("{message}")]
    Generic {
        message: String,
        code: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemErrorStrategy {
    Disable,
    Ignore,
    Retry,
}

pub struct SystemErrorContext {
    pub system_id: SystemId,
    pub error: String, // Rust errors are traits, simplified to String for context
    pub phase: String,
    pub consecutive_failures: u32,
}

pub type SystemErrorHandler = fn(&SystemErrorContext) -> SystemErrorStrategy;

pub fn default_error_handler(ctx: &SystemErrorContext) -> SystemErrorStrategy {
    eprintln!(
        "System {} failed in phase {}: {} (failures: {})",
        ctx.system_id, ctx.phase, ctx.error, ctx.consecutive_failures
    );

    if ctx.consecutive_failures >= 3 {
        eprintln!("System {} disabled due to excessive failures", ctx.system_id);
        return SystemErrorStrategy::Disable;
    }

    SystemErrorStrategy::Ignore
}
