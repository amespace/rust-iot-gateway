// tcp服务

use std::{ net::SocketAddr, sync::Arc, time::Duration};

use futures::{SinkExt, StreamExt};
use tokio::{io::{AsyncRead, AsyncWrite}, net::{TcpListener, TcpStream}};
use tokio_util::codec::Framed;
use tracing::{debug, info, warn};

use crate::{codec::FrameCodec, tls::TlsProvider};
/// TLS 握手超时：防半开连接 / 慢握手
const TLS_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

// shutdown 异步函数 支持跨线程移动 不返回值 不借用外部变量  生产 serve("0.0.0.0:9000", shutdown).await
pub async fn serve(
    addr: &str,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| anyhow::anyhow!("TCP 接入端口{addr} 绑定失败：{e}"))?;
    info!(addr, "Tcp 接入服务已经启动");
    run(listener, shutdown).await
}

pub async fn run(
    listener: TcpListener,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    let mut shutdown = Box::pin(shutdown);

    loop {
        //  同时等待多个异步事件，谁先完成就执行谁的分支。
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream , peer)) => {
                        tokio::spawn(handle_connection(stream, peer)); // 创建一个异步任务 丢进线程池共享执行
                    }
                    Err(e) => {
                        warn!("accept error :{e}")
                    }
                }
            }
            _ = &mut shutdown  => {
                info!("TCP 服务接受到停机信号");
                break;
            }

        }
    }
    Ok(())
}

pub async fn serve_tls(
    addr: &str,
    tls: Arc<TlsProvider>,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| anyhow::anyhow!("TCP 接入端口{addr} 绑定失败：{e}"))?;
    info!(addr, "Tcp 接入服务已经启动（TLS）");
    run_tls(listener, tls, shutdown).await
}

pub async fn run_tls(listener: TcpListener, tls: Arc<TlsProvider> , shutdown: impl Future<Output = ()> + Send + 'static) -> anyhow::Result<()>{
    let mut shutdown = Box::pin(shutdown);
    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, peer)) => {
                        let acceptor = tls.accept(); // 现取当前证书配置（热加载生效点）
                        tokio::spawn(async move {
                            match tokio::time::timeout(
                                TLS_HANDSHAKE_TIMEOUT,
                                acceptor.accept(stream),
                            ).await {
                                Ok(Ok(io)) => handle_io(io, peer).await,
                                Ok(Err(e)) => warn!(%peer, "TLS 握手失败: {e}"),
                                Err(_)     => warn!(%peer, "TLS 握手超时，断开"),
                            }
                        });
                    }
                    Err(e) => {
                        warn!("accept error :{e}")
                    }
                }
            }
            _ = &mut shutdown => {
                info!("TCP 服务（TLS）接受到停机信号");
                break;
            }
        }
    }
    Ok(())
}


// tokio task
async fn handle_connection(stream: TcpStream, peer: SocketAddr) {
    handle_io(stream, peer).await;
}
// 连接处理主体：泛型化，明文流与 TLS 流共用
async fn handle_io<S>(io: S, peer: SocketAddr)
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    info!(%peer, "新连接");
    let mut framed = Framed::new(io, FrameCodec);   // ← 原来这里是 Framed::new(stream, ...)

    while let Some(item) = framed.next().await {
        match item {
            Ok(frame) => {
                // todo!() 暂时先原样返回
                if let Err(e) = framed.send(frame).await {
                    debug!(%peer, "回包失败：{e}");
                    break;
                }
            }
            Err(e) => {
                warn!(%peer, "解码失败，断开: {e}");
                break;
            }
        }
    }
    info!(%peer, "连接关闭");
}