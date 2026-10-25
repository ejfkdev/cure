#!/usr/bin/env python3
"""CPython 3.15 语法编译检查：stdin JSONL {id,cured} → stdout JSONL。"""
import json
import sys

for raw in sys.stdin:
    if not raw.strip():
        continue
    try:
        job = json.loads(raw)
    except Exception as e:
        print(json.dumps({"id": "?", "status": "badjob", "detail": str(e)}, separators=(",", ":")), flush=True)
        continue
    ident = job.get("id", "?")
    try:
        compile(job["orig"], "<orig>", "exec")
    except Exception:
        print(json.dumps({"id": ident, "status": "origerr"}, separators=(",", ":")), flush=True)
        continue
    try:
        compile(job["cured"], "<cured>", "exec")
        print(json.dumps({"id": ident, "status": "ok"}, separators=(",", ":")), flush=True)
    except SyntaxError as e:
        print(json.dumps({
            "id": ident,
            "status": "syntax",
            "detail": f"{e.msg} (line {e.lineno}, col {e.offset})",
        }, separators=(",", ":")), flush=True)
    except Exception as e:
        print(json.dumps({"id": ident, "status": "error", "detail": f"{type(e).__name__}: {e}"}, separators=(",", ":")), flush=True)
