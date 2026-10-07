package com.puppycrawl.tools.checkstyle.checks.modifier.interfacememberimpliedmodifier;

public interface InputInterfaceMemberImpliedModifierMethodsOnInterface4 {
    public static void methodPublicStatic() {}
    static void methodStatic() {}
    public default void methodPublicDefault() {}
    default int methodDefault() {
        return 6;
    }
    public abstract void methodPublicAbstract();
    abstract void methodAbstract();
    public void methodPublic();
    void method();
}
