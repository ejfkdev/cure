package com.puppycrawl.tools.checkstyle.checks.javadoc.missingjavadocmethod;

public class InputMissingJavadocMethodJavadocInMethod {
    public void foo1() {}
    @Deprecated // violation 'Missing a Javadoc comment.'
    public void foo2() {}
    @Deprecated // violation 'Missing a Javadoc comment.'
    /** */
    public void foo3() {}
    public void foo4() {}
    @Deprecated // violation 'Missing a Javadoc comment.'
    public void foo5() {}
    @Deprecated // violation 'Missing a Javadoc comment.'
    /** */
    public void foo6() {}
    public void foo7() {}
    @Deprecated
    public void foo8() {}
    @Deprecated
    /** */
    public void foo9() {}
}
