sealed class Shape {
  const Shape();
}

base class Circle extends Shape {
  final double radius;
  const Circle(this.radius);
}

interface class Named {
  String get name;
}

final class Square extends Shape implements Named {
  final double side;
  const Square(this.side);
  @override
  String get name => 'square';
}

extension type Id(int value) {
  int get doubled => value * 2;
}

enum Planet {
  earth(1),
  mars(2);

  final int order;
  const Planet(this.order);
}

typedef Point = ({int x, int y});
typedef Pair<T> = (T, T);

String describe(Object value) => switch (value) {
      Circle(radius: var r) when r > 1 => 'big circle $r',
      Circle(radius: var r) => 'circle $r',
      Square(side: var s) => 'square $s',
      Id(value: var v) => 'id $v',
      [int a, int b] => 'list $a $b',
      (int a, int b) => 'record $a $b',
      {'name': String n} => 'map $n',
      _ => 'other',
    };

void main() {
  var p = (1, 2);
  var (a, b) = p;
  var named = (x: 1, y: 2);
  var pair = (1, 'two');
  var point = (x: 1, y: 2);
  print(a + b);
  print(named.x + point.y);
  print(pair.$1);
  print(describe(const Circle(2)));
  print(describe(const Square(3)));
  print(describe(Id(4)));
  print(describe([1, 2]));
  print(describe((3, 4)));
  print(describe({'name': 'x'}));
  if (p case (int x, int y) when x < y) {
    print('ordered $x $y');
  }
  var value = switch (a) {
    1 => 'one',
    > 10 => 'big',
    _ => 'other',
  };
  print(value);
  print(Planet.earth.order);
  print(const Id(5).doubled);
}

// Additional Dart 3 pattern families.
String patternZoo(Object? value) => switch (value) {
      null => 'null',
      0 => 'zero',
      int n when n > 0 => 'positive $n',
      int n => 'negative $n',
      String s => 'string $s',
      [int a, int b] => 'pair $a $b',
      [int a, ...] => 'list starting $a',
      {'x': int x} => 'map x=$x',
      (int a, int b) => 'record $a $b',
      Circle(radius: var r) when r > 0 => 'circle $r',
      _ => 'other',
    };

void patternStatements(List<(int, int)> pairs, Object? value) {
  if (value case int n when n > 0) {
    print(n);
  }
  if (value case > 10) {
    print('big');
  }
  if (value case 1 || 2 || 3) {
    print('small');
  }
  if (value case var x?) {
    print(x);
  }
  for (final (a, b) in pairs) {
    print(a + b);
  }
  switch (value) {
    case [int a, int b]:
      print('switch $a $b');
      break;
    case {'x': int x}:
      print('switch map $x');
      break;
    default:
      print('other');
  }
}
