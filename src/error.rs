use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("认证失败")]  
    AuthFailed,  
    #[error("token 过期")]  
    TokenExpired,  
    #[error("限流")]  
    RateLimited,  
    #[error("帧超限 (max {max} bytes)")]  
    FrameTooLarge { max: usize },  
    #[error("重复连接，旧连接被踢")]  
    DuplicateConnection,  
    #[error("服务端关闭")]  
    Shutdown,  
    #[error("协议版本不支持: v{0}")]  
    UnsupportedVersion(u8),  
    #[error("坏帧: {0}")]  
    BadFrame(String),

}

impl ProtocolError {
    // 对应Error 帧里的错误码
    pub const fn code(&self) -> u8 {
        match self {
            Self::AuthFailed => 0x10,  
            Self::TokenExpired => 0x11,  
            Self::RateLimited => 0x12,  
            Self::FrameTooLarge { .. } => 0x13,  
            Self::DuplicateConnection => 0x14,  
            Self::Shutdown => 0x15,  
            Self::UnsupportedVersion(_) => 0x20,  
            Self::BadFrame(_) => 0x21,

        }
    }

    // 判断是否断开连接
    pub fn is_fatal(&self) -> bool {
        matches!(
            self,
            Self::AuthFailed | Self::TokenExpired | Self::Shutdown | Self::UnsupportedVersion(_)

        )
    }
}
