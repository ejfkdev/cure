package com.github.javaparser.ast;

import java.util.EnumSet;

public enum Modifier {
    PUBLIC("public"), PROTECTED("protected"), PRIVATE("private"), ABSTRACT("abstract"), STATIC("static"), FINAL("final"), TRANSIENT("transient"), VOLATILE("volatile"), SYNCHRONIZED("synchronized"), NATIVE("native"), STRICTFP("strictfp");
    String lib;
    private Modifier(String lib) {
        this.lib = lib;
    }
    public String getLib() {
        return lib;
    }
    public EnumSet<Modifier> toEnumSet() {
        return EnumSet.of(this);
    }
    public static AccessSpecifier getAccessSpecifier(EnumSet<Modifier> modifiers) {
        return modifiers.contains(Modifier.PUBLIC) ? AccessSpecifier.PUBLIC : modifiers.contains(Modifier.PROTECTED) ? AccessSpecifier.PROTECTED : modifiers.contains(Modifier.PRIVATE) ? AccessSpecifier.PRIVATE : AccessSpecifier.DEFAULT;
    }
}
