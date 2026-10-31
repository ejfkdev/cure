#!/usr/bin/env python3
"""Dart rule-focused differential fuzzer.

Emits a self-contained program that exercises every simplification rule,
cures it with the CLI, and compares stdout/exit status of the original and
cured program under a given Dart SDK. The program keeps a call log so that
short-circuit/evaluation-order changes are visible.

Usage:
    python3 scripts/dart_rules_fuzz.py <dart-bin> [rounds] [modern]

`modern` is `1` (default) to also emit null-safety/Dart-3 pattern sections;
pass `0` for Dart 1 (no null safety, no patterns).
"""
import os
import random
import subprocess
import sys
import tempfile

CURE = os.environ.get("CURE", "target/release/cure")


COMMON = r"""
var log = [];

void mark(String s) {
  log.add(s);
}

int ti(int tag, int v) {
  mark('i' + tag.toString());
  return v;
}

bool tb(int tag, bool v) {
  mark('b' + tag.toString());
  return v;
}

String consts() {
  var a = 1 + 2 * 3;
  var b = -5 % 3;
  var c = 5 % -3;
  var d = 1 << 4;
  var e = 'a' + 'b' + 'c';
  var f = 'abc'.length;
  var g = 'abc'.indexOf('b');
  var h = 'abc'.contains('bc');
  var i = 'abc'.codeUnitAt(1);
  var j = (255).toRadixString(16);
  var k = (-7).abs();
  var l = (8).bitLength;
  var m = !(1 < 2);
  var n = (1 < 2) ? 'y' : 'n';
  var o = (true ? 1 : 2) + (false ? 3 : 4);
  var p = 1 + 2 < 4 ? 'lt' : 'ge';
  var q = -5 ~/ 3;
  var r = 'abc'.startsWith('ab');
  var s = 'abc'.endsWith('bc');
  var t = 'a😀'.length;
  var u = (42).toString();
  var v = true.toString();
  var w = (-3).isNegative;
  var x = (4).isEven;
  var y = (5).isOdd;
  var z = (3).sign;
  return '$a,$b,$c,$d,$e,$f,$g,$h,$i,$j,$k,$l,$m,$n,$o,$p,$q,$r,$s,$t,$u,$v,$w,$x,$y,$z';
}

String grouping() {
  var x = 10;
  var y = 3;
  var a = x - 1 + 2;
  var b = x + 1 * 2;
  var c = -1 % 3;
  var d = x == 10 + 1;
  var e = true && (x == 10) || true;
  var f = false && (x == 10) || (y == 3);
  var g = (x == 10) && true;
  var h = (x == 10) || false;
  var i = !(x == 10);
  var j = !(x is int);
  var k = !(x is! int);
  var l = !((x == 10));
  var m = 1 + 2 < 4;
  var n = 10 << 1 + 2;
  var o = (7 & 3) == 3;
  return '$a,$b,$c,$d,$e,$f,$g,$h,$i,$j,$k,$l,$m,$n,$o';
}

String tern(int c) {
  log.clear();
  var a = tb(1, c != 0) ? true : false;
  var b = tb(2, c != 0) ? false : true;
  var d = !tb(3, c != 0) ? false : true;
  var e = tb(4, c != 0) ? (tb(5, true) || false) : (false && tb(6, true));
  var f = true ? tb(7, true) : tb(8, false);
  var g = false ? tb(9, true) : tb(10, true);
  return '${a ? 1 : 0}${b ? 1 : 0}${d ? 1 : 0}${e ? 1 : 0}${f ? 1 : 0}${g ? 1 : 0}:${log.join(",")}';
}

String logic(int c) {
  log.clear();
  var a = true && tb(1, c != 0);
  var b = false || tb(2, c != 0);
  var d = false && tb(3, true);
  var e = true || tb(4, false);
  var f = tb(5, c != 0) && true;
  var g = tb(6, c != 0) || false;
  var h = tb(7, c != 0) && false;
  var i = tb(8, c != 0) || true;
  return '${a ? 1 : 0}${b ? 1 : 0}${d ? 1 : 0}${e ? 1 : 0}${f ? 1 : 0}${g ? 1 : 0}${h ? 1 : 0}${i ? 1 : 0}:${log.join(",")}';
}

String stmts(int c) {
  log.clear();
  var x = 0;
  if (c > 0) { x = x + 5; } else { x = x + 7; }
  x = x * 2;
  if (c > 0) { if (c > 1) { x = x + 3; } }
  if (c > 1) { } else { x = x + 11; }
  if (c > 2) { } else if (c > 1) { x = x + 13; }
  if (c > 100) { x = x + 17; }
  mark('x' + x.toString());
  return x.toString() + ':' + log.join(",");
}

String loops(int c) {
  var n = 0;
  for (var i = 0; i < c; i++) { n = n + i; continue; }
  while (c > 0) { n = n + c; c = c - 1; continue; }
  do { n = n + 1; continue; } while (false);
  return n.toString();
}

String tries(int c) {
  var out = '';
  try { out = out + 'a'; } catch (e) { out = out + 'X'; } finally { out = out + 'f'; }
  try { out = out + 'b'; } catch (e) { rethrow; }
  try {
    if (c == 0) { throw 'boom'; }
    out = out + 'c';
  } catch (e) {
    out = out + 'C';
  } finally { }
  try { out = out + 'd'; } finally { }
  if (c < 0) { return out + 'neg'; }
  return out;
}

String rethrow_path(int c) {
  try {
    if (c == 0) { throw 'E0'; }
    return 'ok';
  } catch (e) {
    rethrow;
  }
}

String trailing() {
  var s = 'tr';
  return s;
}

void dead_code() {
  print('alive');
  return;
  print('dead');
}

void trailing_void() {
  print('tv');
  return;
}

String calls(int c) {
  return tir(c);
}

String tir(int c) {
  var out = '';
  try {
    out = out + 't';
    return out + c.toString();
  } finally {
  }
}

int lp1(int x) { final v = 5; return v + x; }
String lp2() { final a = 'x'; final b = a + a; return b; }
bool lp3(bool c) { final t = true; return t ? c : !t; }
int lp4(int n) {
  final step = 2;
  var sum = 0;
  for (var i = 0; i < n; i++) { sum += step * i; }
  return sum;
}
int lp5(int x) { final y = 3; final f = () => y; return f() + x; }
int lp6(int x) { final v = 4; return v.toString().length + x; }
String lp7(int x) { final v = 'v'; return 'v=$v:$x'; }
int lp8(int x) { final v = 1; int g(int v) => v + 1; return g(x) + v; }
String locals() {
  return '${lp1(1)},${lp2()},${lp3(true) ? 1 : 0}${lp3(false) ? 1 : 0},${lp4(4)},${lp5(2)},${lp6(3)},${lp7(4)},${lp8(5)}';
}
"""

MODERN = r"""
String nulls(int? v) {
  var a = !(v == null);
  var b = v != null ? true : false;
  var c = v != null ? false : true;
  var d = v == null ? true : false;
  var e = v == null ? false : true;
  var f = v!;
  return '${a ? 1 : 0}${b ? 1 : 0}${c ? 1 : 0}${d ? 1 : 0}${e ? 1 : 0}$f';
}

String null_assign(int? v, int fallback) {
  var x = v;
  x = x ?? fallback;
  return x.toString();
}

String patterns(Object v) {
  switch (v) {
    case true && true:
      return 'tt';
    case true || false:
      return 'tf';
    case int x when x > 5:
      return 'big$x';
    case int x:
      return 'int$x';
    default:
      return 'other';
  }
}

String ifcase(Object v) {
  if (v case true && true) { return 'a'; }
  if (v case int x when x > 2) { return 'b$x'; }
  if (v case int x) { return 'i$x'; }
  return 'c';
}

String switch_expr(Object v) {
  return switch (v) {
    true && true => 'a',
    int x when x > 5 => 'b',
    int x => 'i$x',
    _ => 'z',
  };
}

String postfix_bang(int? v) {
  var a = v!;
  var b = (v as int?)!;
  var c = [v][0]!;
  return '$a,$b,$c';
}
"""


def program(seed, modern):
    r = random.Random(seed)
    vals = [-3, -2, -1, 0, 1, 2, 3, 10, 100]
    calls = []
    for i in range(6):
        calls.append(vals[r.randrange(len(vals))])
    body = [COMMON]
    if modern:
        body.append(MODERN)
    body.append("""
void main() {
  print(consts());
  print(grouping());
  print(tern(0));
  print(tern(1));
  print(logic(0));
  print(logic(1));
  print(stmts(-1));
  print(stmts(0));
  print(stmts(1));
  print(stmts(2));
  print(stmts(3));
  print(loops(0));
  print(loops(3));
  print(tries(-1));
  print(tries(0));
  print(tries(1));
  print(rethrow_path(1));
  try { rethrow_path(0); } catch (e) { print(e); }
  print(trailing());
  dead_code();
  trailing_void();
  print(tir(7));
  print(calls(8));
  print(locals());
""")
    if modern:
        body.append("""
  print(nulls(5));
  print(null_assign(null, 9));
  print(null_assign(4, 9));
  print(patterns(true));
  print(patterns(false));
  print(patterns(3));
  print(patterns(9));
  print(patterns('x'));
  print(ifcase(true));
  print(ifcase(false));
  print(ifcase(3));
  print(ifcase(9));
  print(ifcase('x'));
  print(switch_expr(true));
  print(switch_expr(false));
  print(switch_expr(3));
  print(switch_expr(9));
  print(switch_expr('x'));
  print(postfix_bang(5));
""")
    for v in calls:
        body.append("  print('call %d: ${calls(%d)}');" % (v, v))
    body.append("}\n")
    return "\n".join(body)


def run(dart, path, modern):
    if modern:
        args = [dart, "run", path]
    else:
        args = [dart, path]
    p = subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                       timeout=180)
    return p.returncode, p.stdout


def main():
    dart = sys.argv[1]
    rounds = int(sys.argv[2]) if len(sys.argv) > 2 else 20
    modern = (sys.argv[3] if len(sys.argv) > 3 else "1") != "0"
    for seed in range(rounds):
        src = program(seed, modern)
        with tempfile.NamedTemporaryFile("w", suffix=".dart", delete=False) as f:
            f.write(src)
            path = f.name
        cured_path = path + ".cured.dart"
        p = subprocess.run([CURE, "-l", "dart", path], stdout=subprocess.PIPE,
                           stderr=subprocess.PIPE, timeout=180)
        if p.returncode != 0:
            print("cure failed", seed, p.stderr.decode()[:500])
            return 1
        open(cured_path, "wb").write(p.stdout)
        a = run(dart, path, modern)
        b = run(dart, cured_path, modern)
        if a != b:
            print("MISMATCH seed", seed)
            print("orig", a[0], a[1][:2000].decode(errors="replace"))
            print("cured", b[0], b[1][:2000].decode(errors="replace"))
            open("/tmp/rules_fuzz_fail.dart", "w").write(src)
            open("/tmp/rules_fuzz_fail.cured.dart", "wb").write(p.stdout)
            return 1
        print("ok", os.path.basename(dart), "seed", seed, "stdout", len(a[1]))
        os.unlink(path)
        os.unlink(cured_path)
    return 0


if __name__ == "__main__":
    sys.exit(main())