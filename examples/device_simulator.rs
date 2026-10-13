//! 设备模拟器：向网关发送一帧 AuthReq，支持 TLS 加密通道。
//!
//! 用法：
//!     cargo run --example device_simulator            # TLS（默认，需 config.toml 里 tls.enable = true）
//!     cargo run --example device_simulator -- plain   # 明文（需 tls.enable = false）
//!
//! 环境变量：
//!     GATEWAY_ADDR   目标地址，默认 127.0.0.1:9000
//!     GATEWAY_CERT   信任的服务端证书，默认 certs/server.crt

use std::sync::Arc;

use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite};
use futures::{SinkExt, StreamExt};
use tokio_rustls::{
    rustls::{
        self, ClientConfig, RootCertStore,
        pki_types::{CertificateDer, ServerName, pem::PemObject},
    },
    TlsConnector,
};

// 引用库中的模块
use rust_iot_gateway::codec::FrameCodec;
use rust_iot_gateway::frames::{Frame, FrameType, Encoding};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = std::env::var("GATEWAY_ADDR").unwrap_or_else(|_| "127.0.0.1:9000".to_string());
    // 带 -- plain 走明文，默认 TLS
    let use_tls = !std::env::args().any(|a| a == "plain");

    // 构造 AuthReq 帧
    let frame = Frame::new(
        FrameType::AuthReq,
        Encoding::Protobuf,
        Frame::msg_id(),
        &b"test-device-001"[..],
    )?;

    println!("目标地址 : {addr}");
    println!("模式     : {}", if use_tls { "TLS（加密）" } else { "明文 TCP" });

    // 显示原始16进制数据
    let mut raw_bytes = bytes::BytesMut::new();
    frame.encode_to(&mut raw_bytes);
    println!("发送帧   : {} 字节 {:02X?}", raw_bytes.len(), &raw_bytes[..]);
    println!("         类型={:#04x} | msg_id={:#018x} | payload={:?}",
             frame.frame_type as u8, frame.msg_id,
             String::from_utf8_lossy(&frame.payload));

    if use_tls {
        // 1. 安装 crypto provider
        install_ring_provider();

        // 2. 把服务端自签证书加入信任库
        let cert_path =
            std::env::var("GATEWAY_CERT").unwrap_or_else(|_| "certs/server.crt".to_string());
        let mut roots = RootCertStore::empty();
        for cert in CertificateDer::pem_file_iter(&cert_path).map_err(|e| {
            format!("打开证书失败 {cert_path}: {e}（先执行 bash scripts/gen_dev_cert.sh）")
        })? {
            roots.add(cert?)?;
        }

        let config = ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth();

        // 3. 建立 TLS 连接
        let tcp = TcpStream::connect(&addr).await?;
        let tls = TlsConnector::from(Arc::new(config))
            .connect(ServerName::try_from("localhost")?, tcp)
            .await?;
        println!("✅ TLS 握手成功，发送帧...");

        // 4. 使用 FrameCodec 处理帧
        let (reader, writer) = tokio::io::split(tls);
        let mut framed_read = FramedRead::new(reader, FrameCodec);
        let mut framed_write = FramedWrite::new(writer, FrameCodec);

        // 发送帧
        framed_write.send(frame).await?;
        println!("发送完成，等待服务器回复...");

        // 接收回复
        if let Some(resp) = framed_read.next().await {
            let resp = resp?;
            print_resp(&resp);
        } else {
            println!("服务器关闭连接");
        }

        // 优雅关闭
        let _ = framed_write.close().await;
    } else {
        // 明文模式
        let tcp = TcpStream::connect(&addr).await?;
        println!("✅ 已连接，发送帧...");

        let (reader, writer) = tokio::io::split(tcp);
        let mut framed_read = FramedRead::new(reader, FrameCodec);
        let mut framed_write = FramedWrite::new(writer, FrameCodec);

        framed_write.send(frame).await?;
        println!("发送完成，等待服务器回复...");

        if let Some(resp) = framed_read.next().await {
            let resp = resp?;
            print_resp(&resp);
        } else {
            println!("服务器关闭连接");
        }

        let _ = framed_write.close().await;
    }

    Ok(())
}

/// 打印响应帧
fn print_resp(frame: &Frame) {
    // 编码为原始字节
    let mut raw_bytes = bytes::BytesMut::new();
    frame.encode_to(&mut raw_bytes);

    println!("收到回复 : {} 字节", raw_bytes.len());
    println!("原始字节 : {:02X?}", &raw_bytes[..]);
    println!("         类型={:#04x} | 编码={:#04x} | msg_id={:#018x} | payload_len={}",
             frame.frame_type as u8,
             frame.enc as u8,
             frame.msg_id,
             frame.payload.len());
    println!("payload  : {:?}", String::from_utf8_lossy(&frame.payload));
}

/// 安装 ring 后端
fn install_ring_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}
