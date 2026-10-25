#!/usr/bin/env python3
"""CPython 3.15 AST 等价驱动：stdin JSONL {id,orig,cured} → stdout JSONL。

只比较语法结构（include_attributes=False），忽略位置/括号/注释/类型注释。
"""
import ast
import json
import sys

SEP = (",", ":")
_orig_dumps = json.dumps

def _compact(obj, **kw):
    kw.setdefault("separators", SEP)
    return _orig_dumps(obj, **kw)

json.dumps = _compact


def dump(src):
    tree = ast.parse(src, type_comments=False)
    return ast.dump(tree, include_attributes=False)

for raw in sys.stdin:
    if not raw.strip():
        continue
    try:
        job = json.loads(raw)
    except Exception as e:
        print(json.dumps({"id": "?", "status": "badjob", "detail": str(e)}, separators=SEP), flush=True)
        continue
    ident = job.get("id", "?")
    try:
        da = dump(job["orig"])
    except Exception as e:
        print(json.dumps({"id": ident, "status": "origerr", "detail": f"{type(e).__name__}: {e}"}), flush=True)
        continue
    try:
        db = dump(job["cured"])
    except Exception as e:
        print(json.dumps({"id": ident, "status": "curederr", "detail": f"{type(e).__name__}: {e}"}), flush=True)
        continue
    if da == db:
        print(json.dumps({"id": ident, "status": "ok"}, separators=SEP), flush=True)
    else:
        i = 0
        while i < min(len(da), len(db)) and da[i] == db[i]:
            i += 1
        lo = max(0, i - 120)
        print(json.dumps({
            "id": ident,
            "status": "diff",
            "detail": f"@{i}: orig=...{da[lo:i+180]}... cured=...{db[lo:i+180]}...",
        }), flush=True)
