use std::fs;

use anyhow::Ok;
use serde::{Deserialize};


#[derive( Deserialize)]
pub struct AppConfig {
    pub instance_id : String,
    pub log_level : String,
    #[serde(default = "d_tcp_addr")]
    pub tcp_addr: String
}

fn d_tcp_addr() -> String {
    "127.0.0.1:9000".to_string()
}

impl AppConfig {
    // 从Toml 文件中加载 
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let raw = fs::read_to_string(path).map_err(|e| anyhow::anyhow!("读取配置{path}失败 {e}"))?;
        let cfg = toml::from_str(&raw).map_err(|e| anyhow::anyhow!("解析 {path} 失败: {e}"))?;
        
        Ok(cfg)
    }
}