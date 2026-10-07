package com.example.pmdtest;

public class PmdTest {
    private static final int lᵤ = 1;
    private static final int μᵤ = 2;
    public static void main(String[] args) {
        System.out.println(lᵤ + μᵤ);
    }
}

enum CodeSet {
    START_CODE_A('Ë'), START_CODE_B('Ì'), START_CODE_C('Í'), A_TILDE('Ã'), STOP_CODE('Î');
    private final char encoding;
    CodeSet(final char encoding) {
        this.encoding = encoding;
    }
    public char getEncoding() {
        return encoding;
    }
}
