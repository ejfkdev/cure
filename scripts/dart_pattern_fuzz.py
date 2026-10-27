#!/usr/bin/env python3
"""Dart 3 pattern-context differential fuzzer.

Generates switch statements, switch expressions and if-case chains with
logical/relational/declaration/list/map/record patterns and guards, then
compares original vs cured output. The simplifier must never rewrite
patterns (pattern `&&`/`||` are not boolean operators).

Usage: python3 scripts/dart_pattern_fuzz.py <dart-3-bin> [rounds]
"""
import os
import random
import subprocess
import sys
import tempfile

CURE = os.environ.get("CURE", "target/release/cure")


def rel_pattern(r):
    op = r.choice([">", "<", ">=", "<=", "==", "!="])
    return "%s %d" % (op, r.randrange(-2, 8))


def logical_bool_pattern(r, d=0):
    if d >= 2:
        return r.choice(["true", "false"])
    k = r.randrange(4)
    if k == 0:
        return "true && true"
    if k == 1:
        return "true || false"
    if k == 2:
        return "%s && %s" % (r.choice(["true", "false"]), r.choice(["true", "false"]))
    return "%s || %s" % (logical_bool_pattern(r, d + 1), logical_bool_pattern(r, d + 1))


def int_pattern(r, d=0):
    k = r.randrange(6)
    if k == 0:
        return str(r.randrange(-2, 8))
    if k == 1:
        return rel_pattern(r)
    if k == 2:
        return "int x"
    if k == 3:
        return "int x && %s" % rel_pattern(r)
    if k == 4:
        return "%s && %s" % (rel_pattern(r), rel_pattern(r))
    return "%s || %s" % (rel_pattern(r), rel_pattern(r))


def guard(r, var="x"):
    choices = [
        "%s > %d" % (var, r.randrange(-2, 6)),
        "%s < %d" % (var, r.randrange(-2, 6)),
        "%s == %d" % (var, r.randrange(-2, 6)),
        "true && %s > %d" % (var, r.randrange(-2, 6)),
        "%s > 0 || false" % var,
        "!(%s == %d)" % (var, r.randrange(-2, 6)),
        "%s %% 2 == 0" % var,
    ]
    return r.choice(choices)


def obj_pattern(r, d=0):
    k = r.randrange(8)
    if k == 0:
        return "[int x, int y]"
    if k == 1:
        return "[int x, ...]"
    if k == 2:
        return "{'a': int x}"
    if k == 3:
        return "{'a': true && true}"
    if k == 4:
        return "(int x, int y)"
    if k == 5:
        return "(int x, String y)"
    if k == 6:
        return "[int x, {'k': int y}]"
    return "[int x && > 2, ...]"


def obj_value(r):
    return r.choice([
        "[1, 2]", "[]", "[7, 3]", "{'a': 1}", "{'a': true}", "{'a': false}",
        "(1, 2)", "(3, 's')", "'z'", "42", "true", "false",
    ])


def module(seed):
    r = random.Random(seed)
    parts = []
    calls = []
    for i in range(12):
        mode = r.randrange(3)
        if mode == 0:
            arms = [int_pattern(r) for _ in range(r.randrange(2, 4))]
            # switch statement
            body = ["String s%d(int v) {" % i, "  switch (v) {"]
            for j, p in enumerate(arms):
                if "int x" in p:
                    body.append("    case %s when %s:" % (p, guard(r)))
                else:
                    body.append("    case %s:" % p)
                body.append("      return 's%d_%d';" % (i, j))
            body += ["    default:", "      return 's%d_d';" % i, "  }", "}"]
            parts.append("\n".join(body))
            # switch expression
            ebody = ["String e%d(int v) => switch (v) {" % i]
            for j, p in enumerate(arms):
                if "int x" in p:
                    ebody.append("  %s when %s => 'e%d_%d'," % (p, guard(r), i, j))
                else:
                    ebody.append("  %s => 'e%d_%d'," % (p, i, j))
            ebody += ["  _ => 'e%d_d'," % i, "};"]
            parts.append("\n".join(ebody))
            calls += [("s%d" % i, "int"), ("e%d" % i, "int")]
        elif mode == 1:
            arms = [logical_bool_pattern(r) for _ in range(r.randrange(2, 4))]
            body = ["bool b%d(bool v) => switch (v) {" % i]
            for j, p in enumerate(arms):
                body.append("  %s => true," % p if j % 2 == 0 else "  %s => false," % p)
            body += ["  _ => false,", "};"]
            parts.append("\n".join(body))
            calls.append(("b%d" % i, "bool"))
        else:
            arms = [obj_pattern(r) for _ in range(r.randrange(2, 4))]
            body = ["String o%d(Object v) {" % i]
            for j, p in enumerate(arms):
                body.append("  if (v case %s) { return 'o%d_%d'; }" % (p, i, j))
            body += ["  return 'o%d_d';" % i, "}"]
            parts.append("\n".join(body))
            calls.append(("o%d" % i, "obj"))
    main = ["void main() {"]
    for name, typ in calls:
        if typ == "int":
            for v in [-1, 0, 1, 2, 3, 5, 7, 10]:
                main.append("  print('%s(%d)=' + %s(%d));" % (name, v, name, v))
        elif typ == "bool":
            for v in ["true", "false"]:
                main.append("  print('%s(%s)=' + (%s(%s) ? '1' : '0'));" % (name, v, name, v))
        else:
            for val in [obj_value(r) for _ in range(6)]:
                main.append("  print('%s(%s)=' + %s(%s));" % (name, val.replace("'", "\\'"), name, val))
    main.append("}\n")
    parts.append("\n".join(main))
    return "\n".join(parts)


def run(dart, path):
    p = subprocess.run([dart, "run", path], stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=180)
    return p.returncode, p.stdout


def main():
    dart = sys.argv[1]
    rounds = int(sys.argv[2]) if len(sys.argv) > 2 else 10
    for seed in range(rounds):
        src = module(seed)
        with tempfile.NamedTemporaryFile("w", suffix=".dart", delete=False) as f:
            f.write(src)
            path = f.name
        cured_path = path + ".cured.dart"
        p = subprocess.run([CURE, "-l", "dart", path], stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=180)
        if p.returncode != 0:
            print("cure failed", seed, p.stderr.decode()[:500])
            return 1
        open(cured_path, "wb").write(p.stdout)
        a = run(dart, path)
        b = run(dart, cured_path)
        if a != b:
            print("MISMATCH seed", seed)
            print("orig", a[0], a[1][:3000].decode(errors="replace"))
            print("cured", b[0], b[1][:3000].decode(errors="replace"))
            open("/tmp/pattern_fuzz_fail.dart", "w").write(src)
            open("/tmp/pattern_fuzz_fail.cured.dart", "wb").write(p.stdout)
            return 1
        print("ok", os.path.basename(dart), "seed", seed, "stdout", len(a[1]))
        os.unlink(path)
        os.unlink(cured_path)
    return 0


if __name__ == "__main__":
    sys.exit(main())
