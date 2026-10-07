package com.puppycrawl.tools.checkstyle.checks.javadoc.atclauseorder;

import java.io.Serializable;

public class InputAtclauseOrderCorrect4 implements Serializable {
    class InnerClassWithAnnotations4 {
        String method5(String aString) {
            return "null";
        }
        String method6(String aString, int aInt, boolean aBoolean) throws Exception {
            return "null";
        }
    }
    InnerClassWithAnnotations4 anon = new InnerClassWithAnnotations4() {
        /**
         * Some text.
         * @param aString Some text.
         * @return Some text.
         * @deprecated Some text.
         */
        String method5(String aString)
        {
            return "null";
        }

        /**
         * Some text.
         * @param aString Some text.
         * @param aInt Some text.
         * @param aBoolean Some text.
         * @return Some text.
         * @throws Exception Some text.
         * @deprecated Some text.
         */
        String method6(String aString, int aInt, boolean aBoolean) throws Exception
        {
            return "null";
        }
    };
}

enum Foo3 {
}

interface FooIn3 {
}
