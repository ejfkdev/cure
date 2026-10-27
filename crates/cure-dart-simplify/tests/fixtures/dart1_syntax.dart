#!/usr/bin/env dart
library legacy_lib;

import 'dart:async';
import 'dart:math' as math show max;
import 'dart:convert' deferred as convert;
export 'dart:math' show max;

typedef int Comparison(int a, int b);
typedef void Callback<T>(T value);

@proxy
class Proxy {}

abstract class Base<T> {
  const Base(this.value);
  final T value;
  factory Base.none() = Base._;
  Base._() : value = null;
  T get data;
  set data(T v);
  int operator [](int i);
  void operator []=(int i, int v);
  String toString() => 'Base($value)';
}

class Mixin {
  int mix = 1;
}

class Impl extends Base<int> with Mixin {
  int _data = 0;
  Impl() : super(0);
  Impl.named({this._data = 1}) : super(0);
  @override int get data => _data;
  @override set data(int v) { _data = v; }
  @override int operator [](int i) => _data + i;
  @override void operator []=(int i, int v) { _data = v - i; }
  Future<int> load() async {
    await new Future<int>.value(1);
    return _data;
  }
  Stream<int> values() async* {
    yield _data;
    yield* new Stream<int>.fromIterable(<int>[1, 2]);
  }
  Iterable<int> syncValues() sync* {
    yield _data;
  }
  static int add(int a, [int b = 2]) => a + b;
  int named({int a: 1, int b: 2}) => a + b;
  int withFinal(final int a, [final int b = 2]) => a + b;
}

void main() {
  var x = new Impl.named(_data: 2);
  x.data = 3;
  var y = x[1];
  x[1] = y;
  print(x?.data ?? 0);
  print(1 is int ? 'int' : 'other');
  var z = x as Base<int>;
  z..data = 4..toString();
  var sym = #symbol;
  print('${sym} ${x.data}');
  print(Impl.add(1));
  print(x.named(b: 3));
}

// Additional Dart 1 syntax families.

const constList = const [1, 2, 3];
const constMap = const {'a': 1};
int topLevel = 1;
int get topGetter => topLevel;
set topSetter(int v) {
  topLevel = v;
}

typedef void VoidCallback();

class Callable {
  int call(int x) => x + 1;
  noSuchMethod(Invocation invocation) => 0;
}

void labelsAndCatch() {
  outer:
  for (var i = 0; i < 3; i++) {
    for (var j = 0; j < 3; j++) {
      if (j == 1) continue outer;
      if (i == 2) break outer;
    }
  }
  for (var c in ['a', 'b']) {
    print(c);
  }
  try {
    throw new StateError('x');
  } on StateError catch (e) {
    print(e);
    rethrow;
  } finally {
    print('f');
  }
  assert(true);
  print(math.max(1, 2));
  print(convert.json);
}
