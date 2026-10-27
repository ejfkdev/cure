class Point(var int x, var int y);

class Point2(var int x, var int y) {
  this : assert(x >= 0 && y >= 0) {
    print('Point2($x, $y)');
  }
}

class const ConstPoint(final int x, final int y) {
  final int z;
  this : z = x + y;
}

class User({required var String _name});

class Person(final String name, final int age);

class Employee(super.name, super.age, final String role) extends Person;

enum Color(final String hex) {
  red('#FF0000'),
  green('#00FF00'),
  blue('#0000FF'),
}

class Classic {
  double x, y;
  new(this.x, this.y);
  new origin() : x = 0, y = 0;
  factory clone(Classic other) => Classic(other.x, other.y);
  const new zero() : x = 0, y = 0;
}

enum Status { none, running, stopped }

void main() {
  var p = Point(1, 2);
  var p2 = Point2(3, 4);
  var cp = const ConstPoint(1, 2);
  var u = User(name: 'x');
  var e = Employee('n', 1, 'r');
  Status current = .running;
  Point origin = .new(1, 2);
  Point named = .new(0, 0);
  var c = Classic.origin();
  var cloned = Classic.clone(c);
  print('$p $p2 $cp $u $e $current $origin $named $cloned');
  print(switch (current) {
    .none => 'none',
    .running => 'running',
    .stopped => 'stopped',
  });
}
