package com.puppycrawl.tools.checkstyle.grammar;

public class InputJava7TryWithResources {
    public static class MyResource implements AutoCloseable {
        @Override
        public void close() throws Exception {}
    }
    public static void main(String[] args) throws Exception {
        try (MyResource resource = new MyResource()) {}
        try (MyResource resource = new MyResource()) {}
        try (MyResource resource = new MyResource()) {} catch (Exception e) {}
        try (MyResource resource = new MyResource()) {} catch (Exception e) {} catch (Throwable t) {}
        try (MyResource resource = new MyResource(); MyResource resource2 = new MyResource()) {} catch (Exception e) {} catch (Throwable t) {}
        try (MyResource resource = new MyResource(); MyResource resource2 = new MyResource()) {} catch (Exception e) {} catch (Throwable t) {}
        try (MyResource resource = new MyResource()) {}
    }
}
