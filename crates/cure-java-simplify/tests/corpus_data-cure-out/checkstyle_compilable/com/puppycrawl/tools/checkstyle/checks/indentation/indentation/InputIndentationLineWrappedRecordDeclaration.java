package com.puppycrawl.tools.checkstyle.checks.indentation.indentation;

import java.io.IOException;

class InputIndentationLineWrappedRecordDeclaration {
    private interface Interf {
        String sayHello();
    }
    private static record ConcreteRecord(                               //indent:4 exp:4
        String greeting                                                 //indent:8 exp:8
    ) implements Interf {
        @Override                                                       //indent:8 exp:8
    public String sayHello() {
            return greeting();
        }
    }
    private static record ConcreteRecord2(                              //indent:4 exp:4
        String greeting                                                 //indent:8 exp:8
) implements Interf {
        @Override                                                       //indent:8 exp:8
    public String sayHello() {
            return greeting();
        }
    }
    private static String method(String greeting) throws Exception {
        return greeting + "test";
    }
    interface SimpleInterface1 {
        default void method() throws IOException {}
    }
    interface SimpleInterface2 {
        default void method() throws IOException {}
    }
    record SimpleRecord1(                                               //indent:4 exp:4
    ) implements SimpleInterface1 {
        record Inner1(                                                  //indent:8 exp:8
        ) {
        }
    }
    record SimpleRecord2(                                                   //indent:0 exp:4 warn
) implements SimpleInterface2 {
        record Inner2(                                                                       //indent:0 exp:4 warn
) {
        }
    }
}
