#!/usr/bin/env bash
# 依序发布 cure workspace 全部 crate 到 crates.io。
#
# 用法:
#   scripts/publish.sh            # 正式发布（需 CARGO_REGISTRY_TOKEN）
#   scripts/publish.sh --dry-run  # 本地校验打包（不打扰 crates.io）
#
# 行为:
#   1. 校验 tag 版本与 workspace 版本一致（在 tag ref 上发布时）
#   2. 按 workspace 依赖拓扑依序发布：engine → java-ast → java-parser →
#      java-print → java-simplify → cli
#   3. 单 crate 失败自动重试（crates.io 索引传播延迟，最多 5 次/间隔 15s）
set -euo pipefail
cd "$(dirname "$0")/.."

DRY=""
if [[ "${1:-}" == "--dry-run" ]]; then
  DRY="--dry-run"
  echo "==> dry-run 模式（不实际上传）"
fi

# --- tag / workspace 版本一致性（仅 CI tag 发布时校验）---
if [[ "${GITHUB_REF_TYPE:-}" == "tag" ]]; then
  tag_ver="${GITHUB_REF_NAME#v}"
  ws_ver=$(awk '/^\[workspace.package\]/{f=1} f && /^version/{gsub(/"/, "", $3); print $3; exit}' Cargo.toml)
  if [[ "$tag_ver" != "$ws_ver" ]]; then
    echo "错误: tag 版本 ($tag_ver) 与 workspace 版本 ($ws_ver) 不一致" >&2
    echo "请先更新 Cargo.toml [workspace.package] version 并提交，再打 tag" >&2
    exit 1
  fi
fi

# --- 依赖拓扑序（被依赖者在前）---
CRATES=(
  cure-engine
  cure-java-ast
  cure-java-parser
  cure-java-print
  cure-java-simplify
  cure-cli
)

for c in "${CRATES[@]}"; do
  echo "==> cargo publish $DRY -p $c"
  attempt=1
  until cargo publish $DRY -p "$c"; do
    if [[ "$DRY" == "--dry-run" || $attempt -ge 5 ]]; then
      echo "错误: 发布 $c 失败" >&2
      exit 1
    fi
    echo "    第 $attempt 次失败，等待 crates.io 索引传播后重试（15s）…"
    sleep 15
    attempt=$((attempt + 1))
  done
  # 依赖刚发布的新版本需要一点时间对下游可见
  if [[ "$DRY" != "--dry-run" ]]; then
    sleep 5
  fi
done

echo "==> 完成：${#CRATES[@]} 个 crate 全部发布"
