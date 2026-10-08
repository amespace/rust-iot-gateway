use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};



#[derive(Serialize, Deserialize)]
struct Cfg {
    listen: String,

    #[serde(default = "d_max_frame")]
    max_frame: usize,

    tls: Option<TlcCfg>
}


fn d_max_frame() -> usize {64 * 1024}

#[derive(Serialize, Deserialize)]
struct TlcCfg {
    cert: String,
    key: String
}

// 测试serde_json
#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Envelope<'a>{
    device_id: &'a str,
    ts: DateTime<Utc>,
    msg_id: uuid::Uuid,
    payload_enc: u8,
    // payload: &'a [u8]

}


#[cfg(test)]
mod tests {
    use super::*;

    #[test] 
    fn toml_test() {
        let cfg: Cfg = toml::from_str(
                r#"
            listen = "0.0.0.0:9000"
            [tls]
            cert = "/etc/gw/cert.pem"
            key  = "/etc/gw/key.pem"
        "#
        ).unwrap();
        assert_eq!(cfg.max_frame, 64 * 1024);
        assert!(cfg.tls.is_some());
    }

    #[test]
    fn serde_test() {
        let env = Envelope {
            device_id: "dev-1", 
            ts: chrono::Utc::now(),
            msg_id: uuid::Uuid::new_v4(),
            payload_enc: 0x02,
            // payload: &[0x01, 0x02,0x03]
        };
        // 转成字节流
        let json = serde_json::to_vec(&env).unwrap();
        // json 字节数据反序列化
        let back: Envelope = serde_json::from_slice(&json).unwrap();

        assert_eq!(env, back);
    }
}