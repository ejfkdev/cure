public class YieldStatements {
    {
        int yield = 2;
        yield (2);
        yield(a, b);
        yield = switch (e) {
            case 1 -> {
                yield(a, b);
                yield = 2;
                yield (2);
                yield++bar;
                yield--bar;
                yield++;
                yield--;
                yield(2);
                yield = switch (foo) {
                    case 4 -> {
                        yield(5);
                    }
                };
                yield () -> {};
                yield();
                yield (2);
                yield !true;
                yield ~0;
                yield +2;
                yield -2;
                yield --foo;
                yield ++foo;
                yield void.class;
                yield double.class;
                yield float.class;
                yield long.class;
                yield int.class;
                yield short.class;
                yield char.class;
                yield byte.class;
                yield boolean.class;
                yield null;
                yield 0x001;
                yield 004;
                yield 2e74;
                yield 0b01;
                yield 0x4P61;
                yield new Object();
                yield (new Object());
                yield switch(foo) {
                                default -> 4;
                            };
                yield this;
                yield super.field;
                yield this.field;
            }
        };
    }
}
