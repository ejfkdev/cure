#!/usr/bin/env bash
# 统一更新 workspace 与所有 cure-* crate 依赖版本，并刷新 Cargo.lock。
# 用法: scripts/bump-version.sh X.Y.Z
set -euo pipefail
cd "$(dirname "$0")/.."
NEW="${1:?usage: scripts/bump-version.sh X.Y.Z}"
[[ "$NEW" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$ ]] || {
  echo "invalid semver: $NEW" >&2; exit 1;
}
python3 - "$NEW" <<'PY'
import glob, re, sys
new = sys.argv[1]
paths = ["Cargo.toml"] + sorted(glob.glob("crates/*/Cargo.toml"))
for p in paths:
    s = open(p).read()
    if p == "Cargo.toml":
        s = re.sub(r'(?m)^version = "[^"]+"', f'version = "{new}"', s, count=1)
    s = re.sub(
        r'(cure-[a-z0-9-]+\s*=\s*\{[^}]*?\bversion\s*=\s*")[^"]+(")',
        lambda m: m.group(1) + new + m.group(2),
        s,
    )
    open(p, "w").write(s)
PY
cargo check --workspace >/dev/null
echo "bumped workspace and cure-* dependencies to $NEW (Cargo.lock refreshed)"
