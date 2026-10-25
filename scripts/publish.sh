#!/usr/bin/env bash
# 依序发布 cure workspace 全部 crate 到 crates.io。
#
# 用法:
#   scripts/publish.sh            # 正式发布（需 CARGO_REGISTRY_TOKEN）
#   scripts/publish.sh --dry-run  # 本地校验打包（不打扰 crates.io）
#
# 行为:
#   1. 校验 tag 版本与 workspace 版本一致（在 tag ref 上发布时）
#   2. 按 workspace 依赖拓扑依序发布：engine/tree → ts-* / python-* /
#      java-* → cli
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
# 正确拓扑序（Java parser 依赖 print；TS parser 只依赖 ts-ast，ts-print
# 的 dev-dependency 在 parser 发布后才可见。曾漏掉全部 cure-ts-* 导致
# CLI 发布时找不到上游 crate）：
# engine → tree → ts-ast → ts-parser → ts-print → ts-simplify
#        → python-ast → python-parser → python-print → python-simplify
#        → java-ast → java-print → java-parser → java-simplify → cli
CRATES=(
  cure-engine
  cure-tree
  cure-ts-ast
  cure-ts-parser
  cure-ts-print
  cure-ts-simplify
  cure-python-ast
  cure-python-parser
  cure-python-print
  cure-python-simplify
  cure-java-ast
  cure-java-print
  cure-java-parser
  cure-java-simplify
  cure-cli
)

for c in "${CRATES[@]}"; do
  # 幂等：该版本已在线上（重推 tag / 部分发布后续跑）→ 跳过而非失败。
  # 版本取 workspace.package（crate 清单为 version.workspace = true，
  # 曾按 crate 清单 awk 取空值致探测 URL 404、幂等失效——v0.1.0
  # 重跑实锺）
  VER=$(awk '/^\[workspace.package\]/{f=1} f && /^version/{gsub(/"/, "", $3); print $3; exit}' Cargo.toml)
  if [ -z "$DRY" ] && [ -n "$VER" ] && curl -sf -H "User-Agent: publish-check" \
      "https://crates.io/api/v1/crates/$c/$VER" > /dev/null 2>&1; then
    echo "==> $c@$VER 已存在于 crates.io —— 跳过"
    continue
  fi
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
