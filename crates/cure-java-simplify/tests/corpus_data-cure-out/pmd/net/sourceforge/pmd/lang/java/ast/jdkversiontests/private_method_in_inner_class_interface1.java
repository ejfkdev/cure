package net.sourceforge.pmd.lang.java.ast.testdata;

public class PrivateMethodsInInterface1 {
    public interface Interface1 {
        Object FOO = new Object() {
            private void privateMethod() { }
        };
    }
    public interface Interface2 {
        class InnerClass {
            private void privateMethod() {}
        }
    }
    public interface Interface3 {
        enum InnerEnum {
            VALUE;
            private void privateMethod() {}
        }
    }
}
