package com.puppycrawl.tools.checkstyle.grammar.antlr4;

import java.io.Serializable;
import java.util.Collection;
import java.util.Collections;
import java.util.Iterator;
import java.util.List;
import java.util.ListIterator;
import java.util.function.IntFunction;
import com.sun.tools.javac.Main;

public class InputAntlr4AstRegressionUncommon implements MyInterface {
    int x[], y[], z[][];
    public Double[] method1() {
        return null;
    }
    protected <B, Z, A> OtherClass.Q<A,B,Z> m() {
        return null;
    }
}

interface MyInterface {
    Double[] method1();
}

class OtherClass extends InputAntlr4AstRegressionUncommon {
    OtherClass(int z) {
        super();
    }
    class Q<A, B, Z> {
    }
    public <Z, B, A> Q<A, Z, B> m() {
        return null;
    }
    OtherClass(int z, String s) {
        new OtherClass(2);
    }
    class Inner {
        Inner(int x) {}
        Inner() {
            this(2);
            List<String> list = new List<>() {
                @Override
                public int size() {
                    return 0;
                }

                @Override
                public boolean isEmpty() {
                    return false;
                }

                @Override
                public boolean contains(Object o) {
                    return false;
                }

                @Override
                public Iterator<String> iterator() {
                    return null;
                }

                @Override
                public Object[] toArray() {
                    return new Object[0];
                }

                @Override
                public <T> T[] toArray(T[] a) {
                    return null;
                }

                @Override
                public boolean add(String s) {
                    return false;
                }

                @Override
                public boolean remove(Object o) {
                    return false;
                }

                @Override
                public boolean containsAll(Collection<?> c) {
                    return false;
                }

                @Override
                public boolean addAll(Collection<? extends String> c) {
                    return false;
                }

                @Override
                public boolean addAll(int index, Collection<? extends String> c) {
                    return false;
                }

                @Override
                public boolean removeAll(Collection<?> c) {
                    return false;
                }

                @Override
                public boolean retainAll(Collection<?> c) {
                    return false;
                }

                @Override
                public void clear() {

                }

                @Override
                public String get(int index) {
                    return null;
                }

                @Override
                public String set(int index, String element) {
                    return null;
                }

                @Override
                public void add(int index, String element) {

                }

                @Override
                public String remove(int index) {
                    return null;
                }

                @Override
                public int indexOf(Object o) {
                    return 0;
                }

                @Override
                public int lastIndexOf(Object o) {
                    return 0;
                }

                @Override
                public ListIterator<String> listIterator() {
                    return null;
                }

                @Override
                public ListIterator<String> listIterator(int index) {
                    return null;
                }

                @Override
                public List<String> subList(int fromIndex, int toIndex) {
                    return null;
                }
            };
        }
    }
    class OtherInner<E> {
        OtherInner(int w) {}
        <T extends Serializable> OtherInner(T element) {}
        OtherInner() {
            <E>this(2);
        }
    }
    private void killAll() {}
    class SuperClassA {
        protected SuperClassA bs;
        protected SuperClassA s;
        Object s2;
    }
    class SuperClassA2 {
    }
    public class FieldAccessSuper extends SuperClassA {
        private SuperClassA s;
        public void foo() {
            s = new SuperClassA();
            this.s.s2 = new SuperClassA2();
            s.s.s2 = new SuperClassA2();
            super.s = new SuperClassA();
            FieldAccessSuper.this.s = new SuperClassA();
        }
        public class Nested extends SuperClassA {
            SuperClassA a;
            class SubscriptionAdapter {
            }
            final SubscriptionAdapter this$0 = null;
            final Object val$argHolder = null;
            public void foo() {
                s = new SuperClassA();
                bs = new SuperClassA();
                FieldAccessSuper.Nested.super.bs = new SuperClassA();
                FieldAccessSuper.super.s = new SuperClassA();
            }
            Object x = Collections.emptyMap();
            Nested() {
                super();
            }
            int bar() {
                return super.hashCode();
            }
            boolean method() {
                return super.equals(new Object());
            }
        }
    }
}

record TestRecord(int[] x, int[]... y) {
    void foo() {}
}
