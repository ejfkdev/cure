package com.puppycrawl.tools.checkstyle.checks.modifier.annotateddeclarationvisibility;

import org.fest.util.VisibleForTesting;

public class InputAnnotatedDeclarationVisibilityMultipleConfigured {
    @com.google.common.annotations.VisibleForTesting
    protected void allowedProtected() {}
    @Deprecated void allowedPackagePrivate() {}
    @Deprecated
    public void violationPublicMethod1() {}
    @com.google.common.annotations.VisibleForTesting
    protected int allowedField1;
    @com.google.common.annotations.VisibleForTesting
    private int violationPrivateField;
    @com.google.common.annotations.VisibleForTesting
    protected InputAnnotatedDeclarationVisibilityMultipleConfigured() {}
    @VisibleForTesting
    public InputAnnotatedDeclarationVisibilityMultipleConfigured(int x) {}
    @Deprecated
    @VisibleForTesting
    protected void allowedMethod() {}
    @Deprecated
    @VisibleForTesting
    public void violationPublicMethod2() {}
    @com.google.common.annotations.VisibleForTesting
    @SuppressWarnings("unused")
    private void violationPrivateMethod() {}
    @VisibleForTesting
    protected int allowedField2;
    @SuppressWarnings("unused")
    @com.google.common.annotations.VisibleForTesting
    public int violationPublicField;
}
