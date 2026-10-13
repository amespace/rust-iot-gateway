#!/usr/bin/env bash
# 生产环境证书获取脚本 - Let's Encrypt
#
# 前置条件:
#   1. 拥有公网可访问的域名
#   2. 域名已解析到服务器 IP
#   3. 80 端口可访问（用于 HTTP 验证）或已有证书（用于 TLS-ALPN 验证）
#
# 用法:
#   首次申请: bash scripts/gen_prod_cert.sh issue your.domain.com
#   续期:     bash scripts/gen_prod_cert.sh renew

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

CERT_DIR="certs/prod"
EMAIL="${CERT_EMAIL:-admin@example.com}"  # 设置环境变量或修改默认值

# 检查 acme.sh 或 certbot
check_tools() {
    if command -v acme.sh >/dev/null 2>&1; then
        echo "使用 acme.sh"
        ACME_TOOL="acme.sh"
    elif command -v certbot >/dev/null 2>&1; then
        echo "使用 certbot"
        ACME_TOOL="certbot"
    else
        echo "错误: 未找到 acme.sh 或 certbot"
        echo "安装 acme.sh: curl https://get.acme.sh | sh -s email=$EMAIL"
        echo "安装 certbot: apt install certbot 或 yum install certbot"
        exit 1
    fi
}

# 首次申请证书
issue_cert() {
    local domain="$1"
    mkdir -p "$CERT_DIR"

    echo "为域名 $domain 申请证书..."

    if [ "$ACME_TOOL" = "acme.sh" ]; then
        # 使用 acme.sh (推荐，纯 shell 实现)
        acme.sh --issue -d "$domain" --standalone --keylength ec-256

        # 安装到指定目录
        acme.sh --install-cert -d "$domain" --ecc \
            --key-file "$CERT_DIR/server.key" \
            --fullchain-file "$CERT_DIR/server.crt" \
            --reloadcmd "systemctl reload iot-gateway"

    elif [ "$ACME_TOOL" = "certbot" ]; then
        # 使用 certbot
        certbot certonly --standalone -d "$domain" \
            --email "$EMAIL" \
            --agree-tos \
            --non-interactive

        # 复制证书到指定目录
        certbot certificates --cert-name "$domain" \
            | grep "Certificate Path" | awk '{print $3}' \
            | xargs -I {} cp {} "$CERT_DIR/server.crt"

        certbot certificates --cert-name "$domain" \
            | grep "Private Key Path" | awk '{print $4}' \
            | xargs -I {} cp {} "$CERT_DIR/server.key"
    fi

    # 设置权限
    chmod 644 "$CERT_DIR/server.crt"
    chmod 600 "$CERT_DIR/server.key"

    echo "✅ 证书已生成:"
    echo "  证书: $CERT_DIR/server.crt"
    echo "  私钥: $CERT_DIR/server.key"
    echo ""
    echo "更新 config/config.toml:"
    echo "  [tls]"
    echo "  enable = true"
    echo "  cert = \"$CERT_DIR/server.crt\""
    echo "  key = \"$CERT_DIR/server.key\""
}

# 续期证书
renew_cert() {
    echo "续期证书..."

    if [ "$ACME_TOOL" = "acme.sh" ]; then
        acme.sh --renew-all
    elif [ "$ACME_TOOL" = "certbot" ]; then
        certbot renew --quiet
    fi

    echo "✅ 证书已续期"
}

# 主逻辑
check_tools

case "${1:-}" in
    issue)
        if [ -z "${2:-}" ]; then
            echo "用法: $0 issue <domain>"
            echo "示例: $0 issue iot.example.com"
            exit 1
        fi
        issue_cert "$2"
        ;;
    renew)
        renew_cert
        ;;
    *)
        echo "用法:"
        echo "  $0 issue <domain>  - 首次申请证书"
        echo "  $0 renew           - 续期证书"
        exit 1
        ;;
esac
