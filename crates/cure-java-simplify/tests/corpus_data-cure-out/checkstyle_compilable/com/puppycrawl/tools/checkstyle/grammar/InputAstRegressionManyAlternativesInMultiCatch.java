package com.puppycrawl.tools.checkstyle.grammar;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;

public class InputAstRegressionManyAlternativesInMultiCatch {
    public static void main(String[] args) {
        try {
            System.out.println(args[7]);
            InputStream stream = new File("myfile.txt").toURL().openStream();
            foo1();
        } catch (ArrayIndexOutOfBoundsException | IOException | SQLException | SecurityException | OneMoreException e) {}
    }
    private static void foo1() throws RuntimeException, SQLException, OneMoreException {}
    private class SQLException extends Exception {
    }
    private class OneMoreException extends Exception {
    }
}
