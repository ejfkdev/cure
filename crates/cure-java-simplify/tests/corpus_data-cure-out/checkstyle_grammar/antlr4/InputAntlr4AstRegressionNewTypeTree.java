package com.puppycrawl.tools.checkstyle.grammar.antlr4;

import java.awt.*;
import java.lang.annotation.ElementType;
import java.lang.annotation.Target;
import java.lang.reflect.AnnotatedElement;
import java.util.List;
import java.util.Map;
import java.util.TreeSet;
import javax.net.ssl.KeyManager;

public class InputAntlr4AstRegressionNewTypeTree {
    private static class Slot {
        double sin2Phi = 2 * StrictMath.sin(1.618033988749895);
        public int[] ci;
    }
    private static class Transformer {
    }
    private static class Ef {
        Transformer[] transformers = new Transformer[10];
        static boolean forward = true;
    }
    void goodMethod() {
        Slot slot = new Slot();
        slot.ci[5] = 10;
        slot.ci = new int[6];
        double[] cZ = {1.1, 1.2};
        double dnZ = slot.sin2Phi * cZ[1];
        Ef ef = new Ef();
        Transformer transformer = ef.forward ? ef.transformers[ef.transformers.length - 1] : ef.transformers[0];
    }
    void badMethod() {
        Slot slot = new Slot();
        slot.ci[5] = 10;
        slot.ci = new int[6];
        double[] cZ = {1.1, 1.2};
        double dnZ = slot.sin2Phi * cZ[1];
        Ef ef = new Ef();
        Transformer transformer = ef.forward ? ef.transformers[ef.transformers.length - 1] : ef.transformers[0];
    }
    void varargLong(@I String[][]... vararg2) {}
    @SuppressWarnings("unused") void withUpperBound(List<? extends int[][]> list) {}
    private static class SSLSecurity {
        static KeyManager[] truncateArray(KeyManager[] kmaw, KeyManager[] keyManagers) {
            return null;
        }
    }
    void method3(int x) {
        KeyManager[] kmaw = null;
        if (x == 1) {
            kmaw = (javax.net.ssl.KeyManager[]) SSLSecurity.truncateArray(kmaw, new javax.net.ssl.KeyManager[3]);
        }
    }
    enum FPMethodArgs {
        IGNN, IIGNN, GNNII, GNNC;
        public Class<?>[] getMethodArguments(boolean isFPType) {
            Class<?> N = isFPType ? Float.TYPE : Integer.TYPE;
            Class<?> G = isFPType ? Graphics2D.class : Graphics.class;
            switch (this) {
                case IGNN:
                    return new Class<?>[] {Integer.TYPE, G, N, N};
                case IIGNN:
                    return new Class<?>[] {Integer.TYPE, Integer.TYPE, G, N, N};
                case GNNII:
                    return new Class<?>[] {G, N, N, Integer.TYPE, Integer.TYPE};
                case GNNC:
                    return new Class<?>[] {G, N, N, Character.TYPE};
                default:
                    throw new RuntimeException("Unknown method arguments!");
            }
        }
    }
    public static class IntData {
        int[] xorData;
        IntData(Object srcPixel, Object xorPixel) {
            this.xorData = (int[]) srcPixel;
            this.xorData = (int[]) xorPixel;
        }
        protected void xorPixel(Object pixData) {
            int[] dstData = (int[]) pixData;
            for (int i = 0; i < dstData.length; i++) {
                dstData[i] ^= xorData[i];
            }
        }
        Object[] getXorData() {
            return new Object[5];
        }
        void method6() {
            if (getXorData().length == 5 && this.getXorData()[1] != IntData.class || this.getXorData()[5] != IntData.class) {
                System.out.println(getXorData().length);
            }
        }
    }
    protected TreeSet<byte []> sortedPrefixes = createTreeSet();
    TreeSet<byte[]> createTreeSet() {
        return null;
    }
    public Object newInstance(Object[] objects) {
        InputAntlr4AstRegressionNewTypeTree tmpConstructor = null;
        return tmpConstructor.newInstance((Object[]) null);
    }
    public TypeVariable<Class<String>>[] getTypeParameters() {
        ClassRepository info = getGenericInfo();
        return info != null ? (TypeVariable<Class<String>>[]) info.getTypeParameters() : (TypeVariable<Class<String>>[]) new TypeVariable<?>[0];
    }
    class ClassRepository {
        public Object getTypeParameters() {
            return null;
        }
    }
    class TypeVariable<E> {
    }
    private ClassRepository getGenericInfo() {
        return null;
    }
    String[] c, d[];
    String[] e, f[];
    public enum Stooge {
        MOE, CURLY, LARRY
    }
    static void checkArrayTypes1(ArrayTypes at, AnnotatedElement e) {
        if (!(at.cls()[0] == Map.class && at.e()[0] == Stooge.MOE)) {}
    }
    static class ArrayTypes {
        Object[] cls;
        public Stooge[] e() {
            return null;
        }
        Object[] cls() {
            return cls;
        }
        public int[] a() {
            return null;
        }
    }
    int[][] w[][];
    int[][] z[][];
    static class P {
        private int x;
        public void set3(P p) {
            synchronized (this) {
                p.x = get()[0];
            }
        }
        public void set4(P p) {
            synchronized (this) {
                p.x = get()[0];
            }
        }
        private int[] get() {
            return null;
        }
    }
    private @Nullable int array2[][];
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface Nullable {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface I {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface J {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface K {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface L {
}
