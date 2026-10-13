#!/usr/bin/env bash
# 生成开发用自签 TLS 证书（仅限本地调试；生产证书必须由 CA 签发）
#
# 用法：bash scripts/gen_dev_cert.sh
# 无论从哪个目录调用，都生成到 <项目根>/certs/ —— 不依赖当前工作目录

set -euo pipefail

# 关键：以脚本自身位置推导项目根并切过去，
# 这样 certs/ 就不再相对「你敲命令时所在的目录」
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$ROOT_DIR"

CERT_DIR="certs"
CN="${1:-localhost}"

if ! command -v openssl >/dev/null 2>&1; then
    echo "错误：找不到 openssl（Windows 可用 Git for Windows 自带的 /usr/bin/openssl）" >&2
    exit 1
fi

# 这一步是 "No such file or directory" 的根源：openssl 不会自动建目录
mkdir -p "$CERT_DIR"

echo "项目根  : $ROOT_DIR"
echo "openssl : $(openssl version)"
echo "生成中..."

ERRLOG=""
if ! ERRLOG="$(MSYS_NO_PATHCONV=1 openssl req -x509 -newkey rsa:2048 -nodes \
        -keyout "$CERT_DIR/server.key" \
        -out    "$CERT_DIR/server.crt" \
        -days 3650 \
        -subj "/C=CN/ST=Zhejiang/L=Hangzhou/O=iot-gateway-dev/CN=$CN" \
        -addext "subjectAltName=DNS:$CN,IP:127.0.0.1" \
        -addext "basicConstraints=critical,CA:FALSE" \
        -addext "keyUsage=critical,digitalSignature,keyEncipherment" \
        -addext "extendedKeyUsage=serverAuth" 2>&1 >/dev/null)"; then
    echo "openssl 执行失败：" >&2
    echo "$ERRLOG" >&2
    exit 1
fi

echo "---"
echo "已生成："
openssl x509 -in "$CERT_DIR/server.crt" -noout -subject -dates
openssl x509 -in "$CERT_DIR/server.crt" -noout -ext subjectAltName
echo "私钥首行：$(head -1 "$CERT_DIR/server.key")"
echo "---"
echo "config/config.toml 需与此一致："
echo "  cert = \"$CERT_DIR/server.crt\""
echo "  key  = \"$CERT_DIR/server.key\""
