use std::{
    collections::HashMap,
    net::SocketAddr,
    thread::{self},
    time::Duration,
};

use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use rumqttd::{Broker, Config, ConnectionSettings, RouterConfig, ServerSettings};
use tokio::time::sleep;

#[tokio::test]
async fn main() {
    let port = 18830;
    let mut cfg = Config {
        id: 0,
        router: RouterConfig {
            max_connections: 100,               // 最大连接数
            max_outgoing_packet_count: 200, // 最大出站包数， 每个连接可以缓存200个待发送的消息包
            max_segment_size: 10 * 1024 * 1024, // 10MB
            max_segment_count: 10,
            custom_segment: None,
            initialized_filters: None,
            shared_subscriptions_strategy: Default::default(), // 多个订阅者共享一个topic的分发策略
        },
        v4: None,
        v5: None,
        ws: None,
        cluster: None,
        console: None,
        bridge: None,
        prometheus: None,
        metrics: None,
    };

    let mut v4_servers = HashMap::new();
    v4_servers.insert(
        "v4-1".to_owned(),
        ServerSettings {
            name: "v4-1".to_owned(),
            listen: format!("127.0.0.1:{port}").parse::<SocketAddr>().unwrap(),
            tls: None,
            next_connection_delay_ms: 1,
            connections: ConnectionSettings {
                connection_timeout_ms: 60000,
                max_payload_size: 10 * 1024,
                max_inflight_count: 10,
                auth: None,
                dynamic_filters: true,
                external_auth: None,
            },
        },
    );
    cfg.v4 = Some(v4_servers);

    let mut broker = Broker::new(cfg);
    thread::spawn(move || broker.start().unwrap());

    sleep(Duration::from_millis(200)).await;
    println!("Broker started on port {port}");

    // 2. 创建订阅者客户端
    let mut mqttoptions = MqttOptions::new("subscriber-1", "127.0.0.1", port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    let ( client, mut connection) = AsyncClient::new(mqttoptions, 10);

    tokio::spawn(async move {
        loop {
            match connection.poll().await {
                // broker收到的消息
                Ok(Event::Incoming(Packet::Publish(publish))) => {
                    println!(
                        "[Subscriber] 收到消息: topic={}, payload={}",
                        publish.topic,
                        String::from_utf8_lossy(&publish.payload)
                    );
                }
                Ok(Event::Incoming(Packet::PubAck(_))) => {
                    println!("[Publisher] 发布确认 (PUBACK)");
                }
                Ok(Event::Incoming(_)) => {}
                // 客户端发出去的消息
                Ok(Event::Outgoing(_)) => {}
                Err(e) => {
                    println!("[Subscriber] 错误: {e}");
                    break;
                }
            }
        }
    });
    sleep(Duration::from_millis(500)).await;

    //3. 订阅topic
    let topic = "test/hello";
    client
        .subscribe(topic, rumqttc::QoS::AtMostOnce)
        .await
        .unwrap();
    sleep(Duration::from_millis(200)).await;

    // 4. 创建发布者客户端
    let mut pub_opts = MqttOptions::new("publisher-1", "127.0.0.1", port);
    pub_opts.set_keep_alive(Duration::from_secs(5));
    let (pub_client, mut pub_eventloop) = AsyncClient::new(pub_opts, 10);
    tokio::spawn(async move {
        loop {
            match pub_eventloop.poll().await {
                Ok(Event::Incoming(Packet::ConnAck(_))) => {
                    println!("[Publisher] 连接成功");
                }
                Ok(Event::Incoming(Packet::PubAck(_))) => {
                    println!("[Publisher] 发布确认 (PUBACK)");
                }
                Ok(Event::Incoming(_)) => {}
                Ok(Event::Outgoing(_)) => {}
                Err(e) => {
                    println!("[Publisher] 错误: {e}");
                    break;
                }
            }
        }
    });
    sleep(Duration::from_millis(500)).await;
    let payload = "Hello Mqtt from Rust";
    pub_client
        .publish(topic, QoS::AtMostOnce, false, payload)
        .await
        .unwrap();

    println!("[Publisher] 已发布: topic={topic}, payload={payload}");
    sleep(Duration::from_secs(2)).await;
    sleep(Duration::from_secs(2)).await;
}
