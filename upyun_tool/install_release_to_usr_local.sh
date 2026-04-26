#!/bin/sh
# 在目标机（如 moicen）AuthCore 仓库根的上级目录执行，或先 cd 到 AuthCore 再：
#   sh upyun_tool/install_release_to_usr_local.sh
set -e
ROOT=$(cd "$(dirname "$0")/.." && pwd)
cd "$ROOT"
cargo build -p upyun_tool --release
sudo cp "$ROOT/target/release/upyun_tool" /usr/local/bin/upyun_tool
echo "Installed /usr/local/bin/upyun_tool"
