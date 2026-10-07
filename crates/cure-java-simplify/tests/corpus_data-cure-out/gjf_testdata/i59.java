import org.checkerframework.checker.tainting.qual.*;

class Outer {
    class Nested {
        class InnerMost {
            @A Outer context(@B Outer.Nested.InnerMost this) {
                return Outer.this;
            }
        }
    }
}
