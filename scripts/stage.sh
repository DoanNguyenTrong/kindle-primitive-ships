#!/bin/sh
# stage.sh [armhf|armel|RUST_TARGET] [STAGE_DIR]
set -eu
cd "$(dirname "$0")/.."
case "${1:-armhf}" in
    armhf|armv7-unknown-linux-musleabihf) target=armv7-unknown-linux-musleabihf ;;
    armel|armv7-unknown-linux-musleabi) target=armv7-unknown-linux-musleabi ;;
    *) echo "Unsupported package architecture: $1" >&2; exit 1 ;;
esac
stage=${2:-build/stage}
target_dir=${CARGO_TARGET_DIR:-target}
bin=$target_dir/$target/release/ships-kindle
[ -x "$bin" ] || { echo "Build $bin before staging." >&2; exit 1; }
install -d "$stage/ships/bin" "$stage/documents"
install -m 755 "$bin" "$stage/ships/bin/ships-kindle"
install -m 755 deploy/Ships.sh "$stage/documents/Ships.sh"
