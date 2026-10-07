public class InputUnusedLocalVariableNoPackageStatement {
    int a = 12;
    void method() {
        int a = 1;
        new InputUnusedLocalVariableNoPackageStatement() {
                    void method() {
                        a += 1;
                    }
                }.getClass();
    }
    void anotherMethod() {
        int var1 = 12;
        int var2 = 13;
        new Foo() {
            void method() {
                var2 += var1;
            }
        }.getClass();
    }
}

class SharkFoo {
    int var1 = 12;
}

class Foo {
    int var2 = 13;
    class SharkFoo {
        int var3 = 13;
        void method() {
            int var3 = 13;
            int var1 = 12;
            new SharkFoo() {
                void method() {
                    var3 += var1;
                }
            }.getClass();
        }
    }
}
