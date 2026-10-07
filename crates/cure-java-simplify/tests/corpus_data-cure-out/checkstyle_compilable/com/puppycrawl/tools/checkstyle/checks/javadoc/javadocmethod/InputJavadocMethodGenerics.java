package com.puppycrawl.tools.checkstyle.checks.javadoc.javadocmethod;

public class InputJavadocMethodGenerics<E extends java.lang.Exception,
                           RE extends RuntimeException & java.io.Serializable> {
    public void method1() throws E {}
    public void method2() throws RE {}
    public void method3() throws E, RE {}
    public <NPE extends NullPointerException> void method4() throws NPE, RE {}
    public class InnerClass<RuntimeException extends ClassCastException> {
        public void method1() throws RuntimeException, RE, java.lang.RuntimeException {}
    }
    public interface InnerInterface<T, E2 extends Throwable> {
        public abstract String doStuff(T t) throws E2;
    }
    public interface InvalidParameterInJavadoc<T> {
    }
    <T> T method(T value) {
        return value;
    }
}
