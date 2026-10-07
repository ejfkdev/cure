package com.google.googlejavaformat;

import com.google.common.base.MoreObjects;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.ImmutableMap;
import com.google.common.collect.ImmutableRangeMap;

public abstract class Input extends InputOutput {
    public interface Tok {
        int getIndex();
        int getPosition();
        int getColumn();
        String getText();
        String getOriginalText();
        int length();
        boolean isNewline();
        boolean isSlashSlashComment();
        boolean isSlashStarComment();
        boolean isJavadocComment();
        boolean isComment();
    }
    public interface Token {
        Tok getTok();
        ImmutableList<? extends Tok> getToksBefore();
        ImmutableList<? extends Tok> getToksAfter();
    }
    public abstract ImmutableList<? extends Token> getTokens();
    public abstract ImmutableRangeMap<Integer, ? extends Token> getPositionTokenMap();
    public abstract ImmutableMap<Integer, Integer> getPositionToColumnMap();
    public abstract String getText();
    public abstract int getkN();
    public abstract Token getToken(int k);
    @Override
  public String toString() {
        return MoreObjects.toStringHelper(this).add("super", super.toString()).toString();
    }
    public abstract int getLineNumber(int inputPosition);
    public abstract int getColumnNumber(int inputPosition);
    public FormatterDiagnostic createDiagnostic(int inputPosition, String message) {
        return new FormatterDiagnostic(getLineNumber(inputPosition), getColumnNumber(inputPosition), message);
    }
}
