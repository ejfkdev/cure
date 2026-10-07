package com.puppycrawl.tools.checkstyle.checks.blocks.leftcurly;

import org.junit.jupiter.api.function.ThrowingConsumer;
import java.net.InetSocketAddress;
import java.net.Socket;
import java.util.*;

public class InputLeftCurlyCommentBeforeLeftCurly2 {
    private long[] countList;
    void method1() {}
    void method2() {
        if (!Arrays.equals(this.countList, countList)) {}
    }
    void method3() {}
    InputLeftCurlyCommentBeforeLeftCurly2() {}
    InputLeftCurlyCommentBeforeLeftCurly2(int data) {}
}

class Class1 {
    private class Node {
        int data;
        Node next;
        Node(int data) {
            this.data = data;
        }
    }
    String s = "🧐  🧐";
    private void foo3(String s) {}
    List<ThrowingConsumer<InetSocketAddress>> targets = List.of((a) -> {
    Socket s = new Socket();
}, (b) -> {
    Socket s = new Socket();
});
}

class Nothing {
    void method() {}
    public void test2(String... para) {}
    public void test3(String line) {
        int index = 0;
        if (line.regionMatches(index, "/**", 0, 3)) {
            index += 2;
        } else if (line.regionMatches(index, "*/", 0, 2)) {
            index++;
        }
    }
    private boolean isThrowable(String className) {
        try {
            Class<?> clazz = Class.forName(className);
            return Throwable.class.isAssignableFrom(clazz);
        } catch (ClassNotFoundException e) {
            return false;
        }
    }
    void paint(boolean selected, Object parent, String bColor) {
        bColor = selected ? "selected" : parent != null ? "parent" : "default";
    }
}
