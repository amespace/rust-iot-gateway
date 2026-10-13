#!/usr/bin/env bash
# 私有 CA 证书生成脚本
#
# 适用场景: 内网部署，设备可预装根证书
#
# 生成结构:
#   certs/private-ca/
#   ├── ca.crt              # 根证书（预装到设备）
#   ├── ca.key              # 根私钥（妥善保管！）
#   ├── server.crt          # 服务端证书（CA 签发）
#   └── server.key          # 服务端私钥
#
# 用法:
#   1. 创建 CA:    bash scripts/gen_private_ca.sh ca
#   2. 签发证书:   bash scripts/gen_private_ca.sh server your.gateway.local
#   3. 部署时:     将 ca.crt 预装到所有 IoT 设备的信任存储

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

CA_DIR="certs/private-ca"
CA_DAYS=3650        # CA 有效期 10 年
SERVER_DAYS=365     # 服务端证书有效期 1 年

# 创建根 CA
create_ca() {
    mkdir -p "$CA_DIR"

    echo "=== 创建私有根 CA ==="

    # 生成 CA 私钥（加密保护）
    echo "生成 CA 私钥..."
    openssl genrsa -aes256 -out "$CA_DIR/ca.key" 4096

    # 生成 CA 根证书
    echo "生成 CA 根证书..."
    openssl req -new -x509 -days $CA_DAYS \
        -key "$CA_DIR/ca.key" \
        -out "$CA_DIR/ca.crt" \
        -subj "/C=CN/ST=Zhejiang/L=Hangzhou/O=YourCompany/CN=YourCompany IoT Root CA" \
        -addext "basicConstraints=critical,CA:TRUE" \
        -addext "keyUsage=critical,keyCertSign,cRLSign"

    # 设置权限
    chmod 600 "$CA_DIR/ca.key"
    chmod 644 "$CA_DIR/ca.crt"

    echo ""
    echo "✅ 根 CA 已创建:"
    echo "  根证书: $CA_DIR/ca.crt (预装到设备)"
    echo "  根私钥: $CA_DIR/ca.key (妥善保管!)"
    echo ""
    echo "查看证书详情:"
    openssl x509 -in "$CA_DIR/ca.crt" -noout -subject -issuer -dates
    echo ""
    echo "⚠️  重要: ca.key 是信任根，泄露将导致整个 PKI 体系不安全！"
    echo "    建议: 离线保存，使用 HSM 或加密 U 盘存储"
}

# 签发服务端证书
issue_server_cert() {
    local cn="${1:-localhost}"

    if [ ! -f "$CA_DIR/ca.crt" ]; then
        echo "错误: 根 CA 不存在，请先运行: $0 ca"
        exit 1
    fi

    echo "=== 为 $cn 签发服务端证书 ==="

    # 生成服务端私钥
    echo "生成服务端私钥..."
    openssl genrsa -out "$CA_DIR/server.key" 2048

    # 创建 CSR 配置文件
    cat > "$CA_DIR/server.csr.cnf" <<EOF
[req]
default_bits = 2048
prompt = no
default_md = sha256
distinguished_name = dn
req_extensions = v3_req

[dn]
C = CN
ST = Zhejiang
L = Hangzhou
O = YourCompany
CN = $cn

[v3_req]
basicConstraints = CA:FALSE
keyUsage = critical, digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth
subjectAltName = @alt_names

[alt_names]
DNS.1 = $cn
DNS.2 = *.yourcompany.local
IP.1 = 127.0.0.1
EOF

    # 创建扩展配置文件
    cat > "$CA_DIR/v3.ext.cnf" <<EOF
basicConstraints = CA:FALSE
keyUsage = critical, digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth
subjectAltName = @alt_names

[alt_names]
DNS.1 = $cn
DNS.2 = *.yourcompany.local
IP.1 = 127.0.0.1
EOF

    # 生成 CSR
    echo "生成证书签名请求..."
    openssl req -new -key "$CA_DIR/server.key" \
        -out "$CA_DIR/server.csr" \
        -config "$CA_DIR/server.csr.cnf"

    # 用 CA 签发
    echo "使用 CA 签发证书..."
    openssl x509 -req -days $SERVER_DAYS \
        -in "$CA_DIR/server.csr" \
        -CA "$CA_DIR/ca.crt" \
        -CAkey "$CA_DIR/ca.key" \
        -CAcreateserial \
        -out "$CA_DIR/server.crt" \
        -extfile "$CA_DIR/v3_ext.cnf"

    # 清理临时文件
    rm -f "$CA_DIR/server.csr" "$CA_DIR/server.csr.cnf" "$CA_DIR/v3_ext.cnf"

    # 设置权限
    chmod 600 "$CA_DIR/server.key"
    chmod 644 "$CA_DIR/server.crt"

    # 验证证书链
    echo ""
    echo "=== 验证证书链 ==="
    openssl verify -CAfile "$CA_DIR/ca.crt" "$CA_DIR/server.crt"

    echo ""
    echo "✅ 服务端证书已签发:"
    echo "  证书: $CA_DIR/server.crt"
    echo "  私钥: $CA_DIR/server.key"
    echo ""
    echo "查看证书详情:"
    openssl x509 -in "$CA_DIR/server.crt" -noout -subject -issuer -dates -ext subjectAltName
    echo ""
    echo "=== 部署说明 ==="
    echo "1. 服务端配置 (config/config.toml):"
    echo "   [tls]"
    echo "   enable = true"
    echo "   cert = \"$CA_DIR/server.crt\""
    echo "   key = \"$CA_DIR/server.key\""
    echo ""
    echo "2. 设备端: 将 $CA_DIR/ca.crt 加入信任存储"
    echo "   - Linux: 复制到 /etc/ssl/certs/ 并运行 update-ca-certificates"
    echo "   - 嵌入式: 烧录到设备固件的信任根列表"
}

# 主逻辑
case "${1:-help}" in
    ca)
        create_ca
        ;;
    server)
        issue_server_cert "${2:-localhost}"
        ;;
    help|*)
        echo "私有 CA 证书管理工具"
        echo ""
        echo "用法:"
        echo "  $0 ca                    - 创建根 CA (只需运行一次)"
        echo "  $0 server <CN>           - 签发服务端证书"
        echo ""
        echo "示例:"
        echo "  $0 ca                    - 创建根 CA"
        echo "  $0 server gw.prod.local  - 为 gw.prod.local 签发证书"
        echo ""
        echo "生成的文件:"
        echo "  certs/private-ca/ca.crt     - 根证书 (预装到设备)"
        echo "  certs/private-ca/ca.key     - 根私钥 (妥善保管!)"
        echo "  certs/private-ca/server.crt - 服务端证书"
        echo "  certs/private-ca/server.key - 服务端私钥"
        ;;
esac
