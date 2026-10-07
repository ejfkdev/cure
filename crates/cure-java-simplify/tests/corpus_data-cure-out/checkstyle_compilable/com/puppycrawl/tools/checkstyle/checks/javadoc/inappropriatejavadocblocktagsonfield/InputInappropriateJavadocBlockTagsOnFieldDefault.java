package com.puppycrawl.tools.checkstyle.checks.javadoc.inappropriatejavadocblocktagsonfield;

public class InputInappropriateJavadocBlockTagsOnFieldDefault {
    public int validField;
    private String invalidReturn;
    protected int invalidParam;
    public boolean invalidThrows;
    public boolean invalidException;
    public int invalidAuthor;
    public String invalidVersion;
    public boolean invalidUses;
    public int invalidProvides;
    public int noJavadocField = 0;
    public int emptyJavadocField = 1;
    public void method() {}
}
