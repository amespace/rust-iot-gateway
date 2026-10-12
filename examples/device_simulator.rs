use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

// 模拟设备接入网关
const MAGIC: u16  = 0x1A6E;
const VERSION: u8 = 0x01;
const FT_AUTH_REQ: u8 = 0x01;
const ENC_PROTOBUF: u8 = 0x01;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:9000";
    println!("连接到服务器: {}", addr);

    let mut stream = TcpStream::connect(addr).await?;
    println!("已连接,发送认证请求帧...");

    // 构造帧: magic(2B) + version(1B) + type(1B) + encoding(1B) + length(4B) + msg_id(8B) + payload
    let mut frame = Vec::new();

    // 魔数 0x1A6E (大端序)
    frame.extend_from_slice(&MAGIC.to_be_bytes());  // 一次性追加两个字节
    // 版本
    frame.push(VERSION);
    // 类型: AuthReq
    frame.push(FT_AUTH_REQ);
    // 编码: Protobuf
    frame.push(ENC_PROTOBUF);
    // payload 长度 (4B, 大端序)
    let payload = b"test-device-001";
    frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    // msg_id (8B) - 使用 UUID v7 的前 64 位
    let msg_id = uuid::Uuid::now_v7().as_u64_pair().0;
    frame.extend_from_slice(&msg_id.to_be_bytes());
    // payload
    frame.extend_from_slice(payload);

    println!("发送帧: {:02X?}", frame);
    stream.write_all(&frame).await?;
    println!("发送完成,等待服务器回复...");

    // 读取服务器回复
    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await?;
    if n > 0 {
        println!("收到回复 ({} 字节): {:02X?}", n, &buf[..n]);
    } else {
        println!("服务器关闭连接");
    }

    Ok(())
}
