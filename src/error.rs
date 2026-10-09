use thiserror::Error;

#[derive(Error)]
pub enum ProtocolError {
    #[error("认证失败")]
    AuthFailed,
    #[error("token 过期")]
    TokenExpired,
    #[error("帧超限 (max {max}B)")]
    FrameTooLarge { max: usize },
    #[error("重复连接被踢")]
    DuplicateConnection,
    #[error("限流")]
    RateLimited,
    #[error("未实现")]
    NotImplemented,
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("编解码 {0}")]
    Codec(String),
    #[error("Redis: {0}")]
    Redis(String),
    #[error("Kafka: {0}")]
    Kafka(String),
    #[error("TLS: {0}")]
    Tls(String),
    #[error("MQTT: {0}")]
    Mqtt(String),
    #[error("业务: {0}")]
    Domain(#[from] DomainError),
}

impl ProtocolError {
    pub const fn code(&self) -> u8 {
        match self {
            Self::AuthFailed => 0x10,
            Self::TokenExpired => 0x11,
            Self::RateLimited => 0x12,
            Self::FrameTooLarge { max } => 0x13,
            Self::DuplicateConnection => 0x14,
            Self::NotImplemented => 0x15,
            Self::ServerShuttingDown => 0x16,
            _ => 0xFF,
        }
    }
}
