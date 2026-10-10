use std::io;

use tokio_util::codec::{Decoder, Encoder};

use crate::frames::{self, Frame};


// 设置一帧的最大尺寸
const MAX_DECODE_BUFFER: usize = 2  * (frames::HEAD_LEN + frames::MAX_PAYLOAD);

pub struct FrameCodec;


// Frame 解码实现
impl Decoder for FrameCodec {
    type Item = Frame;

    type Error= io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() > MAX_DECODE_BUFFER {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("读缓冲 {} > {MAX_DECODE_BUFFER}（疑似慢连接攻击）", src.len())))
        }

        // 帧头没到，需要继续等
        if src.len() < frames::HEAD_LEN {
            src.reserve(frames::HEAD_LEN - src.len());
            return Ok(None)
        }

        //如果长度达到帧头
        // 解析长度字段 5678 // 0x1A6E 魔数 2B , 版本0x01 1B , 类型1B  参考FrameType ,  编码01/02 1B ,  长度 4B , msg_id 8B , payload NB protobuf/ CBOR
        let len = u32::from_be_bytes([src[5], src[6], src[7], src[8]]) as usize;

        if len > frames::MAX_PAYLOAD {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("帧超限: {len} > {}" ,frames::MAX_PAYLOAD)));
        }

        // payload 没到齐
        let total = frames::HEAD_LEN + len ;
        if src.len() < total {
            src.reserve(total - src.len());
            return Ok(None);
        }
        // 取出完整帧
        let frame_bytes = src.split_to(total).freeze();
        frames::Frame::decode(&frame_bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{e}"))).map(Some)
    }
}

impl Encoder<Frame> for FrameCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Frame, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
       item.encode_to(dst);
       Ok(())

    }
}