// @dart=2.19
library modern2;

import 'dart:async';
import 'dart:async' deferred as asyncLib;
export 'dart:math' show max;

typedef Mapper<T, R> = R Function(T value);
typedef void Handler<T>(T value);

mixin Logging on Object {
  void log(String message) => print(message);
}

extension IntTools on int {
  int get twice => this * 2;
}

enum Color with Logging {
  red,
  green,
  blue;

  String get name => toString().split('.').last;
}

class Base<T extends Object?> with Logging {
  final T? value;
  const Base({this.value});
  Base.named(this.value);
  T? get data => value;
  set data(T? v) {}
}

class Impl extends Base<int> with Logging {
  late final int _late;
  int? nullable;
  Impl(int value) : _late = value, super(value: value);
  Impl.required({required this._late, this.nullable}) : super.named(_late);
  @override
  int? get data => _late;
  @override
  set data(int? v) {
    nullable = v;
  }
  Future<int> load() async {
    await Future<void>.value();
    return _late;
  }
  Stream<int> values() async* {
    yield _late;
    yield* Stream<int>.fromIterable(<int>[1, 2]);
  }
  Iterable<int> syncValues() sync* {
    yield _late;
  }
  int sum([int a = 1, int b = 2]) => a + b;
  int named({required int a, int b = 2}) => a + b;
}

class Sub extends Impl {
  Sub(super.value);
}

void main() {
  var x = Impl.required(_late: 2, nullable: null);
  x.data = 3;
  print(x?.data ?? 0);
  print(x.data?.twice);
  var list = <int?>[1, null, 2];
  var spread = <int>[...?list.whereType<int>()];
  var mapped = <int>[for (var v in spread) if (v > 1) v * 2];
  var named = (int a, {required int b}) => a + b;
  print(mapped);
  print(named(1, b: 2));
  print(Color.red.name);
  var z = x as Base<int>;
  z..data = 4;
  var m = <String, List<int>>{'a': [1, 2]};
  print(m);
  print(const <int>{1, 2, 3});
  print(switchless(x));
}

int switchless(Impl x) {
  switch (x.data) {
    case null:
      return 0;
    case 1:
      return 1;
    default:
      return 2;
  }
}

// Additional Dart 2 syntax families.

class Alias = Base<int> with Logging;
class Callable2 {
  int call(int x) => x;
}

Future<int> sumStream(Stream<int> stream) async {
  var total = 0;
  await for (final value in stream) {
    total += value;
  }
  return total;
}

void labelsAndCatch2() {
  outer:
  for (var i = 0; i < 3; i++) {
    for (var j = 0; j < 3; j++) {
      if (j == 1) continue outer;
      if (i == 2) break outer;
    }
  }
  for (final value in <int>[1, 2]) {
    print(value);
  }
  try {
    throw StateError('x');
  } on StateError catch (e) {
    print(e);
    rethrow;
  } finally {
    print('f');
  }
  assert(true);
  var fn = <T>(T value) => value;
  print(fn(1));
  print(asyncLib);
}
