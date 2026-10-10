

// 这里定义数据帧格式
// 2 + 1 +1+ 1 + 4 + 8B + N = 17B  + N
// 0x1A6E 魔数 , 版本0x01 1B , 类型1B  参考FrameType ,  编码01/02 1B ,  长度 4B , msg_id 8B , payload NB protobuf/ CBOR

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::error::ProtocolError;

// 魔数
pub const MAGIC: u16 = 0x1A6E;
// 版本
pub const VERSION : u8 = 0x01;
pub const HEAD_LEN: usize  = 17;
// payload 上限
pub const MAX_PAYLOAD: usize = 64 * 1024;


#[derive(Clone, Copy)]
pub enum FrameType {
    AuthReq    = 0x01, // 设备 → 网关：认证请求  
    AuthResp   = 0x02, // 网关 → 设备：认证结果  
    Ping       = 0x03, // 设备 → 网关：心跳  
    Pong       = 0x04, // 网关 → 设备：心跳应答  
    DataUp     = 0x05, // 设备 → 网关：上行数据  
    CmdDown    = 0x06, // 网关 → 设备：下行指令  
    Ack        = 0x07, // 设备 → 网关：指令回执  
    Error      = 0x08, // 网关 → 设备：错误帧（payload = 1B 错误码 + 描述）  
    Disconnect = 0x09, // 网关 → 设备：主动断开通知

}

impl FrameType {
    pub fn from_u8(v: u8) -> Option<Self> {
        Some(match v {
            0x01 => Self::AuthReq,  
            0x02 => Self::AuthResp,  
            0x03 => Self::Ping,  
            0x04 => Self::Pong,  
            0x05 => Self::DataUp,  
            0x06 => Self::CmdDown,  
            0x07 => Self::Ack,  
            0x08 => Self::Error,  
            0x09 => Self::Disconnect,  
            _ => return None,

        })
    }
}

// paylaod 编码标识
#[derive(Clone, Copy)]
pub enum Encoding {
    Protobuf =  0x01,
    Cbor = 0x02,
}
impl Encoding {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(Self::Protobuf),  
            0x02 => Some(Self::Cbor),  
            _ => None,

        }
    }
}

// 帧完整数据
pub struct Frame {
    pub frame_type : FrameType, 
    pub enc: Encoding, 
    pub msg_id: u64,
    pub payload: Bytes
}

impl Frame {
    pub fn new (
        frame_type: FrameType,
        enc: Encoding,
        msg_id: u64,
        payload: impl Into<Bytes>
    ) -> Result<Self, ProtocolError> {
        let payload = payload .into();
        if payload.len() > MAX_PAYLOAD {
            return Err(ProtocolError::FrameTooLarge { max: MAX_PAYLOAD });
        }
        Ok(Self { frame_type, enc, msg_id, payload})
    }

    // 生成msg_id
    pub fn msg_id() -> u64 {
        uuid::Uuid::now_v7().as_u64_pair().0
    }

    // 构造错误帧
    pub fn error(err: &ProtocolError, msg_id: u64) -> Self {
        let mut p = BytesMut::new();
        p.put_u8(err.code());
        p.put_slice(err.to_string().as_bytes()); // 字节数组
        Self {
            frame_type: FrameType::Error, 
            enc: Encoding::Protobuf, 
            msg_id,
            payload: p.freeze()
        }
    }

    // 帧的总长度 帧头 + payload
    pub fn total_len(&self) -> usize {
        HEAD_LEN + self.payload.len()
    }


    // 追加到buf
    pub fn encode_to(&self, buf: &mut BytesMut) {
        buf.put_u16(MAGIC);
        buf.put_u8(VERSION);
        buf.put_u8(self.frame_type as u8);
        buf.put_u8(self.enc as u8);
        buf.put_u32(self.payload.len() as u32);
        buf.put_u64(self.msg_id as u64);
        buf.put_slice(&self.payload);
    }
    // 解码一帧完整数据

    pub fn decode(buf: &Bytes) -> Result<Self, ProtocolError> {
        if buf.len() < HEAD_LEN {
            return Err(ProtocolError::BadFrame(format!("数据不足帧头： {} < {HEAD_LEN}", buf.len())));
        }

        let mut src = buf.clone();

        let magic  = src.get_u16();
        if magic != MAGIC {
            return Err(ProtocolError::BadFrame(format!("魔数错误： {magic:#06x}")));
        }

        let version = src.get_u8();
        if version != VERSION {
            return Err(ProtocolError::UnsupportedVersion(version));
        }

        let type_v  = src.get_u8();
        let frame_type = FrameType::from_u8(type_v).ok_or_else(|| ProtocolError::BadFrame(format!("未知帧类型：{type_v:#04x}")))?;


        let enc_v = src.get_u8();
        let enc = Encoding::from_u8(enc_v).ok_or_else(|| ProtocolError::BadFrame(format!("未知编码： {enc_v:#04x}")))?;

        let len = src.get_u32() as usize;
        if len > MAX_PAYLOAD {
            return  Err(ProtocolError::FrameTooLarge { max: MAX_PAYLOAD });
        }

        let msg_id = src.get_u64();

        //检查payload 长度是否满足len
        if src.len() < len {
            return Err(ProtocolError::BadFrame(format!("payload 不完整{}/{}", src.len(), len)));
        }
        // 切走后面长度的payload
        let payload= src.split_to(len);
        Ok(Self { frame_type, enc, msg_id, payload})
    }
}