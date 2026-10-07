package com.puppycrawl.tools.checkstyle.checks.javadoc.javadoctype;

public class InputJavadocTypeNoJavadoc_2<T> {
    public int i1;
    protected int i2;
    int i3;
    private int i4;
    public void foo1() {}
    protected void foo2() {}
    void foo3() {}
    private void foo4() {}
    protected class ProtectedInner_2 {
        public int i1;
        protected int i2;
        int i3;
        private int i4;
        public void foo1() {}
        protected void foo2() {}
        void foo3() {}
        private void foo4() {}
    }
    class PackageInner_2 {
        public int i1;
        protected int i2;
        int i3;
        private int i4;
        public void foo1() {}
        protected void foo2() {}
        void foo3() {}
        private void foo4() {}
    }
    private class PrivateInner_2 {
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

class PackageClass_2 {
    public int i1;
    protected int i2;
    int i3;
    private int i4;
    public void foo1() {}
    protected void foo2() {}
    void foo3() {}
    private void foo4() {}
    public class PublicInner_2 {
        public int i1;
        protected int i2;
        int i3;
        private int i4;
        public void foo1() {}
        protected void foo2() {}
        void foo3() {}
        private void foo4() {}
    }
    protected class ProtectedInner_2 {
        public int i1;
        protected int i2;
        int i3;
        private int i4;
        public void foo1() {}
        protected void foo2() {}
        void foo3() {}
        private void foo4() {}
    }
    class PackageInner_2 {
        public int i1;
        protected int i2;
        int i3;
        private int i4;
        public void foo1() {}
        protected void foo2() {}
        void foo3() {}
        private void foo4() {}
    }
    private class PrivateInner_2 {
        public int i1;
        protected int i2;
        int i3;
        private int i4;
        public void foo1() {}
        protected void foo2() {}
        void foo3() {}
        private void foo4() {}
    }
    class IgnoredName_2 {
        private int logger;
        private static final long serialVersionUID = 0;
    }
    void methodWithTwoStarComment() {}
    protected class ProtectedInner2_2<T> {
    }
}
