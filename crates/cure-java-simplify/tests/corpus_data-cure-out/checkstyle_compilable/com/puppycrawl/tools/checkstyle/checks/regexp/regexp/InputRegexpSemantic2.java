package com.puppycrawl.tools.checkstyle.checks.regexp.regexp;

import java.awt.*;
import java.io.ByteArrayOutputStream;
import java.io.File;

class InputRegexpSemantic2 {
    static {
        Boolean x = new Boolean(true);
    }
    {
        Boolean x = new Boolean(true);
        Boolean[] y = {Boolean.TRUE, Boolean.FALSE};
    }
    Boolean getBoolean() {
        return new Boolean(true);
    }
    void otherInstantiations() {
        ByteArrayOutputStream s = new ByteArrayOutputStream();
        File f = new File("/tmp");
        Dimension dim = new Dimension();
        Color col = new Color(0, 0, 0);
    }
    void exHandlerTest() {
        try {} catch (IllegalStateException emptyCatchIsAlwaysAnError) {} catch (NullPointerException ex) {} catch (ArrayIndexOutOfBoundsException ex) {} catch (NegativeArraySizeException ex) {} catch (UnsupportedOperationException handledException) {
            System.out.println(handledException.getMessage());
        } catch (SecurityException ex) {} catch (StringIndexOutOfBoundsException ex) {} catch (IllegalArgumentException ex) {}
    }
    private static final long IGNORE = 666l + 666L;
}
