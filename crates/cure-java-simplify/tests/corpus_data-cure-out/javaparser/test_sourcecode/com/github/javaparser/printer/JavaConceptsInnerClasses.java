package japa.bdd.samples;

import com.github.javaparser.JavaParser;
import japa.parser.ParseException;
import com.github.javaparser.ast.CompilationUnit;
import org.junit.Ignore;
import java.io.*;
import java.util.*;

@Ignore
@Deprecated
public class JavaConceptsInnerClasses<T extends List<int[]>, X> extends Base implements Serializable {
    public <T, E> JavaConceptsInnerClasses(int x) {
        this.arr[0] = x;
        T val1 = null;
        E val2 = null;
        super.check2(val1, val2);
        boolean b = true;
        boolean y = false;
        class X {

                    int i = 0;

                    public <D> X() {
                    }

                    public void m() {
                    }
                }
                @Deprecated
                final class Y extends X {

                    public Y() {
                        super();
                        JavaConceptsInnerClasses.this.cc = 'c';
                        super.i = 1;
                        Y.super.m();
                    }

                    public Y(int y) {
                        super();
                    }

                    public Y(long x) {
                        this();
                    }
                }
    }
    public <T> JavaConceptsInnerClasses(String str) {}
    private class QWE extends JavaConceptsInnerClasses<List<int[]>, String> {
        @Deprecated
        final int z = 0;
        int i = int - 1;
        public QWE(String... x) {
            <String>super(x[0]);
        }
        public QWE(int... x) {
            super(x[0]);
            i = x[0];
            assert true;
            assert true : 2;
            {
                int iii = 3;
                iii += 3;
            }
            label:
                {
                    int iii = 1;
                }
            switch (i) {
            }
            ll:
                switch (i) {
                    case 1:
                        System.out.println(1);
                        break ll;
                    default:
                        {
                            System.out.println("default");
                            break;
                        }
                    case 2:
                        if (t instanceof Base) {
                            System.out.println(1);
                        }
                        i++;
                        ++i;
                }
        }
        private synchronized int[] doSomething() {
            List<? extends Number> x = new ArrayList<Integer>();
            return new int[] {1};
        }
    }
}

class Base {
    public <A, B> void check2(A val1, B val2) {}
}

interface XXX extends Serializable, Cloneable {
}
