package com.puppycrawl.tools.checkstyle.checks.indentation.commentsindentation;

public class InputCommentsIndentationCommentsAfterMethodCall {
    public static void main() {
        InputCommentsIndentationCommentsAfterMethodCall.flatMap(4, 5).flatMap().flatMap();
    }
    public static void main1() {
        InputCommentsIndentationCommentsAfterMethodCall.flatMap(4, 5).flatMap().flatMap();
    }
    public static InputCommentsIndentationCommentsAfterMethodCall flatMap(int i, int h) {
        return new InputCommentsIndentationCommentsAfterMethodCall();
    }
    public static InputCommentsIndentationCommentsAfterMethodCall flatMap() {
        return new InputCommentsIndentationCommentsAfterMethodCall();
    }
}
