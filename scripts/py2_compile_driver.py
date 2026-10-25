#!/usr/bin/env python2
# -*- coding: utf-8 -*-
"""CPython 2.7 编译检查驱动：stdin JSONL {id,orig,cured} → stdout JSONL。"""
import json
import sys

SEP = (",", ":")


def emit(obj):
    sys.stdout.write(json.dumps(obj, separators=SEP) + "\n")
    sys.stdout.flush()


def code(src):
    return src.encode("utf-8")


for raw in sys.stdin:
    if not raw.strip():
        continue
    try:
        job = json.loads(raw)
    except Exception as e:
        emit({"id": "?", "status": "badjob", "detail": str(e)})
        continue
    ident = job.get("id", "?")
    try:
        compile(code(job["orig"]), "<orig>", "exec")
    except Exception:
        emit({"id": ident, "status": "origerr"})
        continue
    try:
        compile(code(job["cured"]), "<cured>", "exec")
        emit({"id": ident, "status": "ok"})
    except SyntaxError as e:
        emit({"id": ident, "status": "syntax",
              "detail": "%s (line %s, col %s)" % (e.msg, e.lineno, e.offset)})
    except Exception as e:
        emit({"id": ident, "status": "error",
              "detail": "%s: %s" % (type(e).__name__, e)})