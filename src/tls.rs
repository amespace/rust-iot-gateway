// 加密通道 + 证书热加载

use std::{
    path::{Path, PathBuf}, sync::{Arc, Once, RwLock}, time::{Duration, SystemTime},
};

use anyhow::Context;
use tokio_rustls::{
    TlsAcceptor,
    rustls::{
        self, ServerConfig,
        pki_types::{
            CertificateDer, PrivateKeyDer,
            pem::{PemObject},
        },
    },
};
pub fn install_crypto_provider() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        if rustls::crypto::ring::default_provider()
            .install_default()
            .is_err()
        {
            tracing::debug!("crypto provider 已安装")
        }
    });
}

pub struct TlsProvider {
    slot: RwLock<Arc<ServerConfig>>,
}
impl TlsProvider {
    pub fn from_files(cert: &Path, key: &Path) -> anyhow::Result<Self> {
        Ok(Self {
            slot: RwLock::new(load_server_config(cert, key)?),
        })
    }

    fn current(&self) -> Arc<ServerConfig> {
        // &RwLockReadGuard 这里会自动强制转换为Arc<ServerConfig>
        Arc::clone(&self.slot.read().unwrap_or_else(|e| e.into_inner()))
    }

    pub fn accept(&self) -> TlsAcceptor {
        TlsAcceptor::from(self.current())
    }

    pub fn reload(&self, cert: &Path, key: &Path) -> anyhow::Result<()> {
        let cfg = load_server_config(cert, key)?;
        *self.slot.write().unwrap_or_else(|e| e.into_inner()) = cfg;
        Ok(())
    }

    /// 后台轮询证书文件 mtime，变化则重载；重载失败保留旧证书。
    pub fn spawn_reload_watcher(
        self: &Arc<Self>,
        cert: PathBuf,
        key: PathBuf,
        interval: Duration,
    ) -> tokio::task::JoinHandle<()> {
        let this = Arc::clone(self);
        tokio::spawn(async move {
            let mut last = mtime(&cert);
            loop {
                tokio::time::sleep(interval).await;
                let now = mtime(&cert);
                if now.is_some() && now != last {
                    match this.reload(&cert, &key) {
                        Ok(()) => {
                            last = now;
                            tracing::info!(cert = %cert.display(), "TLS 证书已热加载");
                        }
                        Err(e) => {
                            tracing::warn!("TLS 证书热加载失败，继续使用旧证书: {e:#}");
                        }
                    }
                }
            }
        })
    }
}

// 这里会获取证书的文件元数据
fn mtime(p: &Path) -> Option<SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

pub fn load_server_config(cert: &Path, key: &Path) -> anyhow::Result<Arc<ServerConfig>> {
    let chain: Vec<CertificateDer<'static>> = CertificateDer::pem_file_iter(cert)
        .with_context(|| format!("打开证书失败: {}", cert.display()))?
        .collect::<Result<Vec<_>, _>>()
        .with_context(|| format!("解析证书 PEM 失败: {}", cert.display()))?;

    let priv_key = PrivateKeyDer::from_pem_file(key)
        .with_context(|| format!("解析私钥 PEM 失败: {}", key.display()))?;
    let cfg = ServerConfig::builder()
        .with_no_client_auth() // 设备身份走应用层 AuthReq 帧（S9），暂不用 mTLS
        .with_single_cert(chain, priv_key)
        .context("构建 ServerConfig 失败（证书与私钥不匹配？）")?;

    Ok(Arc::new(cfg))
}
