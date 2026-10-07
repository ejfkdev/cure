package com.puppycrawl.tools.checkstyle.grammar.java9;

public class InputJava9TryWithResources {
    public static class MyResource implements AutoCloseable {
        @Override
        public void close() throws Exception {}
    }
    public static void main(String[] args) throws Exception {
        MyResource resource = new MyResource();
        try (resource) {}
        MyResource resource1 = new MyResource();
        MyResource resource2 = new MyResource();
        try (resource1; resource2) {}
    }
}
