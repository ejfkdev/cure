package com.puppycrawl.tools.checkstyle.checks.javadoc.javadocpackage;

public class InputJavadocPackageNoJavadoc {
    public int i1;
    protected int i2;
    int i3;
    private int i4;
    public void foo1() {}
    protected void foo2() {}
    void foo3() {}
    private void foo4() {}
    protected class ProtectedInner {
        public int i1;
        protected int i2;
        int i3;
        private int i4;
        public void foo1() {}
        protected void foo2() {}
        void foo3() {}
        private void foo4() {}
    }
    class PackageInner {
        public int i1;
        protected int i2;
        int i3;
        private int i4;
        public void foo1() {}
        protected void foo2() {}
        void foo3() {}
        private void foo4() {}
    }
    private class PrivateInner {
        public int i1;
        protected int i2;
        int i3;
        private int i4;
        public void foo1() {}
        protected void foo2() {}
        void foo3() {}
        private void foo4() {}
    }
}
