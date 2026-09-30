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
#[derive(Serialize, Deserialize)]
struct Envelope<'a>{
    device_id: &'a str,
    ts: DateTime<Utc>,
    msg_id: uuid::Uuid,
    payload_enc: u8,
    payload: &'a [u8]

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
}