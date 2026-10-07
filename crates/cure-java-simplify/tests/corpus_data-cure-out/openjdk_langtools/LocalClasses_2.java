class LocalClasses_2 {
    void foo() {
        class Local { }
                {
                    class Local { }                     // ERROR
                }
    }
    void bar() {
        class Local { }

                class Baz {
                    void quux() {
                        class Local { }                 // OK
                    }
                }

                class Quux {
                    void baz() {
                        class Random {
                            void quem() {
                                class Local { }         // OK
                            }
                        }
                    }
                }
    }
}
