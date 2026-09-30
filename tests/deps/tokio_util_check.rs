use bytes::{Buf, BufMut, Bytes};
use tokio_util::codec::{Decoder, Encoder};


// 测试协议按照 (payload长度)+ payload
struct DemoFrame { payload : Bytes}


struct DemoCodec;

impl Decoder for DemoCodec {
    type Item = DemoFrame;

    type Error = std::io::Error;

    fn decode(&mut self, src: &mut bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {

        // 解析协议
        if src.len() < 4{ src.reserve(4); return Ok(None);}
        // 8 8 8 8 四个字节  
        let n = u32::from_be_bytes([src[0], src[1], src[2], src[3]]) as usize;
        if src.len() < 4 + n { src.reserve(4 + n - src.len()); return Ok(None);}
        src.advance(4);

        // 取出payload转换成Bytes
        let payload = src.split_to(n).freeze();

        Ok(Some(DemoFrame { payload }))
    }
}

impl Encoder<DemoFrame> for DemoCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: DemoFrame, dst: &mut bytes::BytesMut) -> Result<(), Self::Error> {
        // 编码
        dst.put_u32(item.payload.len() as u32);
        dst.extend_from_slice(&item.payload);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn test_encode() {
        let mut codec = DemoCodec;
        let mut dst = BytesMut::new();

        let frame = DemoFrame {
            payload: Bytes::from("hello"),
        };

        codec.encode(frame, &mut dst).unwrap();

        // 期望: [0x00, 0x00, 0x00, 0x05] + "hello"
        assert_eq!(dst.len(), 4 + 5);
        assert_eq!(&dst[..4], &[0x00, 0x00, 0x00, 0x05]);
        assert_eq!(&dst[4..], b"hello");
    }

    #[test]
    fn test_decode() {
        let mut codec = DemoCodec;
        let mut buf = BytesMut::new();
        buf.extend_from_slice(&[0x00, 0x00, 0x00, 0x05]); // 长度 = 5
        buf.extend_from_slice(b"hello");

        let frame = codec.decode(&mut buf).unwrap().unwrap();

        assert_eq!(frame.payload.as_ref(), b"hello");
        assert!(buf.is_empty());
    }
    
    #[test]
    fn test_encode_decode_roundtrip() {
        let mut codec = DemoCodec;

        // 编码
        let original = DemoFrame {
            payload: Bytes::from("hello world"),
        };
        let mut buf = BytesMut::new();
        codec.encode(original, &mut buf).unwrap();

        // 解码
        let decoded = codec.decode(&mut buf).unwrap().unwrap();

        assert_eq!(decoded.payload.as_ref(), b"hello world");
    }

   
}