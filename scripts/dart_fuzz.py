#!/usr/bin/env python3
"""Dart statement-level differential fuzzer.

Generates Dart 1/2/3 common-subset modules, cures them, and compares the
original and cured program under a given Dart SDK.
"""
import os
import random
import subprocess
import sys
import tempfile

CURE = os.environ.get("CURE", "target/release/cure")

def expr(r, depth=0):
    atoms = ["a", "b", "0", "1", "2", "3", "4"]
    if depth >= 2:
        return r.choice(atoms)
    k = r.randrange(6)
    if k == 0:
        return "(%s + %s)" % (expr(r, depth + 1), expr(r, depth + 1))
    if k == 1:
        return "(%s - %s)" % (expr(r, depth + 1), expr(r, depth + 1))
    if k == 2:
        return "(%s * %s)" % (expr(r, depth + 1), expr(r, depth + 1))
    if k == 3:
        return "(%s %% (%s + 1))" % (expr(r, depth + 1), expr(r, depth + 1))
    if k == 4:
        return "(%s ? %s : %s)" % (
            cond(r, depth + 1),
            expr(r, depth + 1),
            expr(r, depth + 1),
        )
    return "(%s + 1)" % expr(r, depth + 1)

def cond(r, depth=0):
    if depth >= 2:
        return "(a > b)"
    k = r.randrange(5)
    if k == 0:
        return "(a > %s)" % expr(r, depth + 1)
    if k == 1:
        return "(b < %s)" % expr(r, depth + 1)
    if k == 2:
        return "(a == b)"
    if k == 3:
        return "!(a > b)"
    return "((a > 0) && (b >= 0))"

def module(seed):
    r = random.Random(seed)
    out = []
    n = 35
    for i in range(n):
        out.append("int f%d(int a, int b) {" % i)
        body = []
        body.append("  var x = %s;" % expr(r))
        kind = r.randrange(5)
        if kind == 0:
            body += ["  if (%s) {" % cond(r), "    return %s;" % expr(r), "  } else {", "    return %s;" % expr(r), "  }"]
        elif kind == 1:
            body += ["  if (%s) return %s;" % (cond(r), expr(r)), "  return %s;" % expr(r)]
        elif kind == 2:
            body += ["  if (%s) { x = %s; } else { x = %s; }" % (cond(r), expr(r), expr(r)), "  return x;"]
        elif kind == 3:
            body += ["  if (%s) { return %s; }" % (cond(r), expr(r)), "  return %s;" % expr(r)]
        else:
            body += ["  var y = 0;", "  for (var i = 0; i < 2; i++) { y += i; continue; }",
                     "  while (a > 0) { y += a; a = a - 1; continue; }",
                     "  do { y += 1; continue; } while (false);",
                     "  return y + x;"]
        if kind in (0, 1, 2, 3) and r.randrange(2) == 0:
            body += ["  print('dead%d');" % i, "  return -1;"]
        out.extend(body)
        out.append("}")
        out.append("")
    out.append("void main() {")
    for i in range(n):
        for a, b in [(0, 1), (1, 2), (2, 3)]:
            out.append("  print(f%d(%d, %d));" % (i, a, b))
    out.append("}")
    out.append("")
    return "\n".join(out)

def run(dart, path):
    args = [dart, path] if os.environ.get("DART_NO_RUN") == "1" else [dart, "run", path]
    p = subprocess.run(args, stdout=subprocess.PIPE,
                       stderr=subprocess.PIPE, timeout=120)
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
        p = subprocess.run([CURE, "-l", "dart", path], stdout=subprocess.PIPE,
                           stderr=subprocess.PIPE, timeout=120)
        if p.returncode != 0:
            print("cure failed", seed, p.stderr.decode()[:500])
            return 1
        open(cured_path, "wb").write(p.stdout)
        a = run(dart, path)
        b = run(dart, cured_path)
        if a != b:
            print("MISMATCH seed", seed)
            print("orig", a[0], a[1][:300])
            print("cured", b[0], b[1][:300])
            open("/tmp/fuzz_dart_fail.dart", "w").write(src)
            open("/tmp/fuzz_dart_fail.cured.dart", "wb").write(p.stdout)
            return 1
        print("ok", os.path.basename(dart), "seed", seed, "stdout", len(a[1]))
        os.unlink(path)
        os.unlink(cured_path)
    return 0

if __name__ == "__main__":
    sys.exit(main())