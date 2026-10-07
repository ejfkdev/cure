package com.puppycrawl.tools.checkstyle.checks.coding.unnecessarysemicolonaftertypememberdeclaration;

public record InputUnnecessarySemicolonAfterTypeMemberDeclarationRecords() {
    static {}
    static {}
    public InputUnnecessarySemicolonAfterTypeMemberDeclarationRecords {}
    public InputUnnecessarySemicolonAfterTypeMemberDeclarationRecords(Object o) {
        this();
    }
    void method() {}
    static int field = 10;
    static {}
    static {}
    void anotherMethod() {
    }
}
