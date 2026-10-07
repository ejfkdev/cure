package com.puppycrawl.tools.checkstyle.checks.blocks.emptycatchblock;

import java.io.IOException;

public class InputEmptyCatchBlockCommentRecognitionLF {
    private void some() {
        try {
            throw new IOException();
        } catch (IOException e) {}
    }
    private void some1() {
        try {
            throw new IOException();
        } catch (IOException e) {}
    }
    private void some2() {
        try {
            throw new IOException();
        } catch (IOException e) {}
    }
    private void some3() {
        try {
            throw new IOException();
        } catch (IOException e) {}
    }
    private void some4() {
        try {
            throw new IOException();
        } catch (IOException e) {}
    }
    private void some5() {
        try {
            throw new IOException();
        } catch (IOException e) {}
    }
    private void emptyMultilineComment() {
        try {
            throw new IOException();
        } catch (IOException e) {}
    }
}
