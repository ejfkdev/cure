package com.puppycrawl.tools.checkstyle.checks.coding.onestatementperline;

import java.awt.event.ActionEvent;
import java.lang.annotation.Annotation;
import java.lang.String;
import java.lang.Integer;
import java.util.LinkedList;
import java.util.List;
import java.util.concurrent.Callable;
import java.util.stream.Collectors;
import javax.swing.JCheckBox;

public class InputOneStatementPerLine {
    static {
        new JCheckBox().addActionListener((final ActionEvent e) -> {
            good();
        });
        List<Integer> ints = new LinkedList<Integer>();
        ints.stream().map((t) -> {
            return t * 2;
        }).filter((t) -> {
            return false;
        });
        ints.stream().map((t) -> {
            return t * 2;
        });
        ints.stream().map((t) -> {
            return t * 2;
        });
        ints.stream().map((t) -> t * 2);
        ints.stream().map((t) -> t * 2);
        List<Integer> ints2 = new LinkedList<Integer>();
        ints.stream().map((t) -> {
            return ints2.stream().map((w) -> {
                return w * 2;
            });
        });
        ints.stream().map((t) -> {
            return ints2.stream().map((w) -> {
                return w;
            });
        });
        ints.stream().map((t) -> {
            return ints2.stream().map((w) -> {
                return w * 2;
            });
        });
        ints.stream().map((t) -> {
            int l = 0;
            for (int j = 0; j < 10; j++) {
                l = j + l;
            }
            return l;
        });
    }
    private static void good() {}
    InputOneStatementPerLine method(foo a) {
        foo obj = () -> {
            method(() -> {
                method(null);
            }).method(null);
        };
        return this;
    }
    interface foo {
        void method();
    }
}
