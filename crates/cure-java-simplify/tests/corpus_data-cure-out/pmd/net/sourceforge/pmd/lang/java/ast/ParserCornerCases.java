class Superclass {
    public Superclass() {}
    public <V> Superclass(Class<V> clazz) {}
    <T> T doStuff(T s) {
        return s;
    }
}

class Outer {
    Outer() {
        System.out.println("Outer constructor");
    }
    class Inner {
        Inner() {
            System.out.println("Inner constructor");
        }
    }
}

class Child extends Outer.Inner {
    Child(Outer o) {
        o.super();
        System.out.println("Child constructor");
    }
}

public class ParserCornerCases extends Superclass {
    public ParserCornerCases() {
        super();
    }
    public ParserCornerCases(int a) {
        <Integer> this(a, 2);
    }
    public <W> ParserCornerCases(int a, int b) {
        <String> super(String.class);
    }
    public ParserCornerCases(String title) {
        this();
    }
    public strictfp void testGeneric() {
        String o = super.doStuff("foo");
        String v = this.thisGeneric("bar");
    }
    <X> X thisGeneric(X x) {
        return x;
    }
    Class getByteArrayClass() {
        return byte[].class;
    }
    public void bitwiseOperator() {
        if ((modifiers & InputEvent.SHIFT_DOWN_MASK) != 0) {
            buf.append("shift ");
        }
    }
}

class PmdTestParent {
    public PmdTestParent(Object obj) {}
}

class PmdTestChild extends PmdTestParent {
    public PmdTestChild() {
        super(new Object() {

			public Object create() {

				Object memoryMonitor = null;

				if (memoryMonitor == null) {
					memoryMonitor = new Object();
				}

				return memoryMonitor;
			}
		});
    }
}

class SimpleBean {
    String name;
}

class SimpleBeanUser {
    SimpleBeanUser(SimpleBean o) {}
    SimpleBeanUser() {
        this(new SimpleBean() {{
            name = "test";
        }});
    }
}

class SimpleBeanUser2 extends SimpleBeanUser {
    SimpleBeanUser2() {
        super(new SimpleBean() {{
            name = "test2";
        }});
    }
}

class TestParseAnnototation {
    void parse() {
        for (int i = 0; i < 10; i++) {}
        for (Iterator it = Fachabteilung.values().iterator(); it.hasNext(); ) {}
        List<String> l = new ArrayList<String>();
        for (String s : l) {}
    }
}

class FooBlock {
}

class MyFoo {
    MyFoo(FooBlock b) {}
}

class Foo extends MyFoo {
    public Foo() {
        super(new FooBlock() {
            public Object valueOf(Object object) {
                String fish = "salmon";
                return fish;
            }
        });
    }
}

class SuperTest {
    public Iterator<E> iterator() {
        if (this.mods.contains(Modification.Iterator)) {
            return new Iterator<E>() {
                Iterator<E> wrapped = ImmutableSet.super.iterator();

                public boolean hasNext() {
                    return this.wrapped.hasNext();
                }

                public E next() {
                    return this.wrapped.next();
                }

                public void remove() {
                    if (ImmutableSet.this.mods.contains(Modification.RemoveIter)) {
                        this.wrapped.remove();
                    }
                    throw new UnsupportedOperationException();
                }
            };
        }
        throw new UnsupportedOperationException();
    }
}

class ClazzPropertyOfPrimitiveTypes {
    public void test() {
        if ("a".equals(int.class.getName())) {}
        if (Integer.class.equals(clazz) || int.class.equals(clazz)) {}
    }
}
