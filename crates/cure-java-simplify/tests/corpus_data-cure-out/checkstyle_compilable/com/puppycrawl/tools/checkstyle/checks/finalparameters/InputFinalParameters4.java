package com.puppycrawl.tools.checkstyle.checks.finalparameters;

import java.awt.event.ActionEvent;
import javax.swing.AbstractAction;
import javax.swing.Action;

class InputFinalParameters4 {
    InputFinalParameters4() {}
    InputFinalParameters4(String s) {}
    InputFinalParameters4(final Integer i) {}
    InputFinalParameters4(final @MyAnnotation33 Class<Object> i) {}
    InputFinalParameters4(@MyAnnotation33 Boolean i) {}
    InputFinalParameters4(String s, final Integer i) {}
    void method() {}
    void method(String s) {}
    void method(final Integer i) {}
    void method(@MyAnnotation33 final Object s) {}
    void method(@MyAnnotation33 Class<Object> s) {}
    void method(String s, final Integer i) {}
    interface TestInterface {
        void method(String s);
    }
    void holder() {
        Action a = new AbstractAction() {
                public void actionPerformed(ActionEvent e)
                {
                }
                void somethingElse(@MyAnnotation33 ActionEvent e)
                {
                }
            };
        Action b = new AbstractAction() {
                public void actionPerformed(final ActionEvent e)
                {
                }
                void somethingElse(@MyAnnotation33 final ActionEvent e)
                {
                }
            };
    }
}
