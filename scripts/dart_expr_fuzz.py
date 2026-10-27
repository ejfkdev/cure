#!/usr/bin/env python3
"""Typed random-expression differential fuzzer for the Dart simplifier.

Generates valid Dart 1/2/3 programs mixing typed int/bool/String expressions,
nested ifs, empty branches, loops, compound assignments and try/finally
control flow. The original and cured programs are executed and their
stdout/exit status compared.

Usage: python3 scripts/dart_expr_fuzz.py <dart-bin> [rounds]
Dart 1: DART_NO_RUN=1 python3 scripts/dart_expr_fuzz.py <dart-1-bin> ...
"""
import os
import random
import subprocess
import sys
import tempfile

CURE = os.environ.get("CURE", "target/release/cure")
VARS = ["a", "b"]
INTS = ["0", "1", "2", "3", "4", "5", "7", "10"]
STRS = ["''", "'a'", "'abc'", "'x y'", "'0'"]


def i_expr(r, d=0):
    if d >= 3:
        return r.choice(INTS + VARS[:3])
    k = r.randrange(12)
    if k < 4:
        return r.choice(INTS + VARS[:3])
    if k == 4:
        return "(%s + %s)" % (i_expr(r, d + 1), i_expr(r, d + 1))
    if k == 5:
        return "(%s - %s)" % (i_expr(r, d + 1), i_expr(r, d + 1))
    if k == 6:
        return "(%s * %s)" % (i_expr(r, d + 1), i_expr(r, d + 1))
    if k == 7:
        return "(%s ~/ %d)" % (i_expr(r, d + 1), r.choice([2, 3, 4]))
    if k == 8:
        return "(%s %% %d)" % (i_expr(r, d + 1), r.choice([3, 5, 7]))
    if k == 9:
        return "(%s << %d)" % (i_expr(r, d + 1), r.randrange(0, 4))
    if k == 10:
        return "(-%s)" % i_expr(r, d + 1)
    return "(%s ? %s : %s)" % (b_expr(r, d + 1), i_expr(r, d + 1), i_expr(r, d + 1))


def b_expr(r, d=0):
    if d >= 3:
        return r.choice(["true", "false", "(a > b)", "(a == 0)"])
    k = r.randrange(12)
    if k < 2:
        return r.choice(["true", "false", "(a > b)", "(a == 0)", "(b < 3)"])
    if k == 2:
        return "(%s == %s)" % (i_expr(r, d + 1), i_expr(r, d + 1))
    if k == 3:
        return "(%s != %s)" % (i_expr(r, d + 1), i_expr(r, d + 1))
    if k == 4:
        return "(%s < %s)" % (i_expr(r, d + 1), i_expr(r, d + 1))
    if k == 5:
        return "(%s >= %s)" % (i_expr(r, d + 1), i_expr(r, d + 1))
    if k == 6:
        return "(!%s)" % b_expr(r, d + 1)
    if k == 7:
        return "(%s && %s)" % (b_expr(r, d + 1), b_expr(r, d + 1))
    if k == 8:
        return "(%s || %s)" % (b_expr(r, d + 1), b_expr(r, d + 1))
    if k == 9:
        return "(%s ? %s : %s)" % (b_expr(r, d + 1), b_expr(r, d + 1), b_expr(r, d + 1))
    if k == 10:
        return "(!(%s == %s))" % (i_expr(r, d + 1), i_expr(r, d + 1))
    return "(!(%s is int))" % i_expr(r, d + 1)


def s_expr(r, d=0):
    if d >= 2:
        return r.choice(STRS)
    k = r.randrange(6)
    if k < 2:
        return r.choice(STRS)
    if k == 2:
        return "(%s + %s)" % (s_expr(r, d + 1), s_expr(r, d + 1))
    if k == 3:
        return "(%s.length.toString())" % s_expr(r, d + 1)
    if k == 4:
        return "((%s).contains('a').toString())" % s_expr(r, d + 1)
    return "(%s ? %s : %s)" % (b_expr(r, d + 1), s_expr(r, d + 1), s_expr(r, d + 1))


def int_fn(r, name, kind):
    x = i_expr(r)
    y = i_expr(r)
    if kind == 0:
        return "int %s(int a, int b) {\n  var x = %s;\n  if (%s) { x = %s; } else { x = %s; }\n  return x;\n}" % (name, x, b_expr(r), i_expr(r), i_expr(r))
    if kind == 1:
        return "int %s(int a, int b) {\n  if (%s) { return %s; }\n  return %s;\n}" % (name, b_expr(r), i_expr(r), i_expr(r))
    if kind == 2:
        return "int %s(int a, int b) {\n  var x = %s;\n  if (%s) { x = %s; }\n  return x;\n}" % (name, x, b_expr(r), i_expr(r))
    if kind == 3:
        return "int %s(int a, int b) {\n  var x = %s;\n  if (%s) { if (%s) { x = %s; } }\n  return x;\n}" % (name, x, b_expr(r), b_expr(r), i_expr(r))
    if kind == 4:
        return "int %s(int a, int b) {\n  var x = %s;\n  if (%s) { } else { x = %s; }\n  return x;\n}" % (name, x, b_expr(r), i_expr(r))
    if kind == 5:
        return "int %s(int a, int b) {\n  var x = %s;\n  var y = %s;\n  for (var i = 0; i < 2; i++) { x = x + %s; continue; }\n  while (b > 0) { x = x + b; b = b - 1; continue; }\n  do { x = x + 1; continue; } while (false);\n  x = x + y;\n  return x;\n}" % (name, x, y, i_expr(r))
    if kind == 6:
        return "int %s(int a, int b) {\n  var x = 0;\n  try {\n    x = x + %s;\n    if (%s) { throw 'E'; }\n    x = x + %s;\n  } catch (e) {\n    rethrow;\n  } finally { }\n  return x;\n}" % (name, i_expr(r), b_expr(r), i_expr(r))
    if kind == 7:
        return "int %s(int a, int b) {\n  var x = %s;\n  try { x = x + %s; } catch (e) { x = x + 100; } finally { }\n  try { x = x + %s; } finally { }\n  return x;\n}" % (name, x, i_expr(r), i_expr(r))
    if kind == 8:
        return "int %s(int a, int b) {\n  var x = %s;\n  x = x + %s;\n  x = x * 2;\n  x = x - %s;\n  return x;\n}" % (name, x, i_expr(r), i_expr(r))
    return "int %s(int a, int b) {\n  var x = %s;\n  if (%s) { } else if (%s) { x = %s; } else { x = %s; }\n  return x;\n}" % (name, x, b_expr(r), b_expr(r), i_expr(r), i_expr(r))


def bool_fn(r, name):
    k = r.randrange(3)
    if k == 0:
        return "bool %s(int a, int b) {\n  if (%s) { return true; } else { return false; }\n}" % (name, b_expr(r))
    if k == 1:
        return "bool %s(int a, int b) {\n  if (%s) { return false; }\n  return true;\n}" % (name, b_expr(r))
    return "bool %s(int a, int b) {\n  var v = %s;\n  return v;\n}" % (name, b_expr(r))


def str_fn(r, name):
    return "String %s(int a, int b) {\n  var s = %s;\n  if (%s) { s = %s; } else { s = %s; }\n  return s;\n}" % (name, s_expr(r), b_expr(r), s_expr(r), s_expr(r))


def program(seed):
    r = random.Random(seed)
    parts = ["var log = [];", "void mark(String s) { log.add(s); }"]
    n = 40
    kinds = []
    for i in range(n):
        name = "f%d" % i
        t = r.randrange(3)
        if t == 0:
            parts.append(int_fn(r, name, r.randrange(10)))
            kinds.append(("int", name))
        elif t == 1:
            parts.append(bool_fn(r, name))
            kinds.append(("bool", name))
        else:
            parts.append(str_fn(r, name))
            kinds.append(("str", name))
    main = ["void main() {"]
    vals = [(0, 0), (1, 2), (-1, 3), (3, 0), (10, 4)]
    for t, name in kinds:
        for a, b in vals:
            if t == "int":
                main.append("  try { print('%s(%d,%d)=' + %s(%d, %d).toString()); } catch (e) { print('%s(%d,%d)=E:' + e.toString()); }" % (name, a, b, name, a, b, name, a, b))
            elif t == "bool":
                main.append("  try { print('%s(%d,%d)=' + (%s(%d, %d) ? '1' : '0')); } catch (e) { print('%s(%d,%d)=E'); }" % (name, a, b, name, a, b, name, a, b))
            else:
                main.append("  try { print('%s(%d,%d)=' + %s(%d, %d)); } catch (e) { print('%s(%d,%d)=E'); }" % (name, a, b, name, a, b, name, a, b))
    main.append("}\n")
    parts.append("\n".join(main))
    return "\n".join(parts)


def run(dart, path):
    args = [dart, path] if os.environ.get("DART_NO_RUN") == "1" else [dart, "run", path]
    p = subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=180)
    return p.returncode, p.stdout


def main():
    dart = sys.argv[1]
    rounds = int(sys.argv[2]) if len(sys.argv) > 2 else 10
    for seed in range(rounds):
        src = program(seed)
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
            open("/tmp/expr_fuzz_fail.dart", "w").write(src)
            open("/tmp/expr_fuzz_fail.cured.dart", "wb").write(p.stdout)
            return 1
        print("ok", os.path.basename(dart), "seed", seed, "stdout", len(a[1]))
        os.unlink(path)
        os.unlink(cured_path)
    return 0


if __name__ == "__main__":
    sys.exit(main())
