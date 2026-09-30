# 生产级别IOT

## 项目结构

```
iot-gateway/
├── Cargo.toml / Cargo.lock
├── README.md / CHANGELOG.md / CONTRIBUTING.md
├── config/
│   ├── config.toml                    # 默认配置
│   ├── config.dev.toml                # 开发环境
│   └── config.prod.toml               # 生产环境
│
├── proto/                             
│   ├── uplink.proto
│   ├── downlink.proto
│   └── auth.proto
│
├── deploy/
│   ├── Dockerfile                      ★ 多阶段构建 + distroless 运行
│   ├── docker-compose.yml              ★ 本地 Kafka/Redis/Prom/Grafana 栈
│   └── k8s/
│       ├── deployment.yaml
│       ├── hpa.yaml
│       ├── pdb.yaml
│       └── servicemonitor.yaml
│
├── docs/
│   ├── protocol.md                     ★ 帧格式 / 错误码 / 版本策略
│   ├── runbook.md                      ★ 告警与处置
│   └── capacity-plan.md                ★ 容量与资源规划
│
├── benches/                            ★ criterion 压测代码
│   ├── codec.rs
│   └── session_table.rs
│
├── examples/                           ★ 可执行的 demo
│   ├── device_simulator.rs             ★ 设备模拟器
│   └── debug_cli.rs
│
├── scripts/                            ★ 运维脚本（不进容器）
│   ├── cert-gen.sh                     ★ 本地自签证书
│   └── kafka-topics.sh                 ★ 初始化 topics
│
├── src/
│   ├── main.rs                         # 二进制入口（极薄，仅 bootstrap + run）
│   ├── lib.rs                          # 库根（集成测试和 examples 复用）
│   ├── bootstrap.rs                    ★ 装配：创建所有组件并连线
│   ├── shutdown.rs                     ★ SIGTERM → drain 30s → 退出
│   ├── error.rs                        # 顶层错误枚举（thiserror）
│   │
│   ├── config/                         # 配置按域分文件
│   │   ├── mod.rs
│   │   ├── app.rs                      # 实例 ID / 日志级别 / 指标路径
│   │   ├── ingress.rs                  # tcp/mqtt 监听口 + TLS 路径
│   │   ├── egress.rs                   # kafka/redis 地址 + topic 模板
│   │   └── limit.rs                    # 限流阈值
│   │
│   ├── telemetry/                      ★ 可观测性独立模块
│   │   ├── mod.rs
│   │   ├── metrics.rs                  # prometheus 注册与导出
│   │   └── logging.rs                  # tracing-subscriber 装配
│   │
│   ├── tls/                            ★ TLS 适配
│   │   ├── mod.rs
│   │   └── reload.rs                   # ArcSwap 热加载
│   │
│   ├── limit/                          ★ 限流（governor）
│   │   ├── mod.rs
│   │   ├── conn.rs                     # 每 IP 连接速率
│   │   └── msg.rs                      # 每设备消息速率
│   │
│   ├── proto/                          ★ prost 生成代码入口
│   │   ├── mod.rs
│   │   └── uplink.rs                   # build.rs 编译期生成
│   │
│   ├── domain/                         ★ 纯业务（无 IO 依赖）
│   │   ├── mod.rs
│   │   ├── device.rs                   # Device 实体 + 状态机
│   │   ├── device_id.rs                # newtype 强类型
│   │   ├── credentials.rs              # Credentials 实体
│   │   └── envelope.rs                 # 上行 envelope 数据结构
│   │
│   ├── ingress/                        ★ 接入层（协议适配器）
│   │   ├── mod.rs
│   │   ├── tcp/
│   │   │   ├── mod.rs
│   │   │   ├── server.rs               # 监听 + accept loop
│   │   │   ├── codec.rs                # tokio-util Framed 编解码
│   │   │   ├── frame.rs                # 帧头定义
│   │   │   └── handler.rs              # 业务回调（解码→auth→投递）
│   │   └── mqtt/
│   │       ├── mod.rs
│   │       ├── broker.rs               # rumqttd 嵌入式 broker
│   │       └── bridge.rs               # 向 EMQX 桥接
│   │
│   ├── auth/                           # 认证服务
│   │   ├── mod.rs
│   │   ├── jwt.rs                      # token 签发与校验
│   │   ├── password.rs                 # argon2 哈希
│   │   └── cache.rs                    # moka TTL 缓存
│   │
│   ├── session/                        ★ 会话管理
│   │   ├── mod.rs
│   │   ├── table.rs                    # dashmap 本地表
│   │   ├── route.rs                    # Redis 路由注册
│   │   └── heartbeat.rs                ★ 心跳维护与 TTL 续期
│   │
│   ├── downlink/                       ★ 下行应用服务
│   │   ├── mod.rs
│   │   ├── consumer.rs                 # 消费下行 topic
│   │   ├── dispatcher.rs               # 路由到具体会话投递
│   │   ├── pending.rs                  # Redis 离线补投队列
│   │   └── ack.rs                      # 回执 topic 投递
│   │
│   ├── egress/                         ★ 出口
│   │   ├── mod.rs
│   │   ├── kafka.rs                    # 上行发布
│   │   └── redis.rs                    # 凭据 / 路由 / 补投 / 状态读写
│   │
│   ├── device_mgmt/                    ★ 设备管理
│   │   ├── mod.rs
│   │   ├── credentials.rs              # 凭据 CRUD（落 Redis）
│   │   └── device_type.rs              # 设备类型注册
│   │
│   └── ops/                            ★ 运维 HTTP
│       ├── mod.rs
│       ├── router.rs                   # axum 路由
│       ├── state.rs                    # 共享状态注入
│       └── handlers.rs                 # healthz/readyz/metrics/sessions/kick/devices
│
└── tests/
    ├── unit/
    │   └── tcp_codec_test.rs           # codec 半包粘包 + proptest 模糊测试
    ├── deps/                           ★ 依赖验证测试
    │   ├── tokio_check.rs
    │   └── tokio_util_check.rs
    ├── integration/                    ★ testcontainers 端到端
    │   ├── kafka_roundtrip.rs
    │   ├── redis_session_route.rs
    │   └── tls_handshake.rs
    └── tls_handshake_test.rs           # rcgen 自签证书握手
```