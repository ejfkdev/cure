#!/usr/bin/env python2
# -*- coding: utf-8 -*-
"""CPython 2.7 AST 等价驱动：stdin JSONL {id,orig,cured} → stdout JSONL。

源文本按 UTF-8 编码成 byte string 再交给 ast.parse，让 PEP 263 编码
声明生效（Python 2 拒绝在 unicode 字符串里出现 encoding declaration）。
"""
import ast
import json
import sys

SEP = (",", ":")


def emit(obj):
    sys.stdout.write(json.dumps(obj, separators=SEP) + "\n")
    sys.stdout.flush()


def dump(src):
    # Python 2 的 ast.parse 需要 str（bytes），编码声明才会被处理。
    tree = ast.parse(src.encode("utf-8"))
    return ast.dump(tree)


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
        da = dump(job["orig"])
    except Exception as e:
        emit({"id": ident, "status": "origerr", "detail": "%s: %s" % (type(e).__name__, e)})
        continue
    try:
        db = dump(job["cured"])
    except Exception as e:
        emit({"id": ident, "status": "curederr", "detail": "%s: %s" % (type(e).__name__, e)})
        continue
    if da == db:
        emit({"id": ident, "status": "ok"})
    else:
        i = 0
        while i < min(len(da), len(db)) and da[i] == db[i]:
            i += 1
        lo = max(0, i - 120)
        emit({
            "id": ident,
            "status": "diff",
            "detail": "@%d: orig=...%s... cured=...%s..." % (
                i, da[lo:i + 180], db[lo:i + 180]),
        })