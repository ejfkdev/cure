package com.puppycrawl.tools.checkstyle.checks.modifier.annotateddeclarationvisibility;

public class InputAnnotatedDeclarationVisibilityBasic {
    @com.google.common.annotations.VisibleForTesting
    protected void allowedProtectedMethod() {}
    @com.google.common.annotations.VisibleForTesting void allowedPackagePrivateMethod() {}
    @com.google.common.annotations.VisibleForTesting
    public void violationPublicMethod() {}
    @com.google.common.annotations.VisibleForTesting
    private void violationPrivateMethod() {}
    @com.google.common.annotations.VisibleForTesting
    protected int allowedProtectedField;
    @com.google.common.annotations.VisibleForTesting
    public int violationPublicField;
    protected InputAnnotatedDeclarationVisibilityBasic() {}
    @com.google.common.annotations.VisibleForTesting
    public InputAnnotatedDeclarationVisibilityBasic(int x) {}
}
