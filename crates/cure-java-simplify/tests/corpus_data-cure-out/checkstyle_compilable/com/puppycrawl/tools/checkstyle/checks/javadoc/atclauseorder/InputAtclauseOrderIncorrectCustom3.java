package com.puppycrawl.tools.checkstyle.checks.javadoc.atclauseorder;

import java.io.Serializable;

class InputAtclauseOrderIncorrectCustom3 implements Serializable {
    class InnerClassWithAnnotations3 {
        void method3() throws Exception {}
        String method4() throws Exception {
            return "null";
        }
    }
    InnerClassWithAnnotations3 anon = new InnerClassWithAnnotations3() {
        /**
         * Some text.
         * @deprecated Some text.
         * @throws Exception Some text.
         */
        void method3() throws Exception {}

        /**
         * Some text.
         * @throws Exception Some text.
         * @return Some text.
         */
        String method4() throws Exception
        {
            return "null";
        }
    };
}
