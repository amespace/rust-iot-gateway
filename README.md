# 生产级别IOT

## 开发证书生成

本地 TLS 调试用的自签证书，一条命令生成。**不引入任何 Rust 依赖**，用系统自带的 `openssl`。

```bash
bash scripts/gen_dev_cert.sh                 # 默认 CN=localhost
bash scripts/gen_dev_cert.sh myhost.local    # 自定义 CN
```

产出：

| 文件 | 说明 |
| --- | --- |
| `certs/server.crt` | 自签证书（PEM），SAN = `DNS:<CN>, IP:127.0.0.1`，有效期 10 年 |
| `certs/server.key` | PKCS#8 私钥（PEM） |

**新项目**：把 `scripts/gen_dev_cert.sh` 拷过去即可（单文件、零依赖），只需改脚本开头的 `CERT_DIR` 和 `-subj` 里的 `O=`。
