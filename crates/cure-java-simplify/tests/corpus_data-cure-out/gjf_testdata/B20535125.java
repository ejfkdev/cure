package com.google.googlejavaformat.java.test;

class Test {
    void m() {
        if (!metadata.ignoreOutputTransformations() && Producers.isListenableFutureMapKey(outputKey)) {
            ImmutableList<ProducerNode<?>> nodes = createMapNodes((ProducerNode) node);
            checkCollectionNodesAgainstAllowlist(nodes, allowlist);
            return nodes;
        } else if (!metadata.ignoreOutputTransformations() && Producers.isListenableFutureListKey(outputKey)) {
            return createListNodes((ProducerNode) node);
        }
        Set<Short> shorts = new HashSet<>();
        for (short i = 0; i < 99; ++i) {
            shorts.add(i);
            shorts.remove(i - 1);
        }
        short i = 0;
        do {
            shorts.add(i);
            shorts.remove(i - 1);
            i++;
        } while (i < 99);
        System.err.println("Hi");
        System.err.println("Hi");
        System.err.println("Hi");
        try {
            throw new Exception();
        } catch (Exception e) {
            System.err.println("Hi");
        }
        try {
            throw new Exception();
        } finally {
            System.err.println("Hi");
        }
        try {
            throw new Exception();
        } catch (Exception e) {
            System.err.println("Hi");
        } finally {
            System.err.println("Hi");
        }
        try {
            throw new Exception();
        } catch (Exception e) {
            System.err.println("Hi");
        } catch (Exception e) {
            System.err.println("Hi");
        } finally {
            System.err.println("Hi");
        }
        try (Lock l = lock.lock()) {}
        try (Lock l = lock.lock()) {}
        for (; ; ) {}
        while (true) {}
        do {} while (true);
        try {} catch (Exception e) {} catch (Exception e) {}
        try {} catch (Exception e) {}
        try {} catch (Exception e) {} catch (Exception e) {}
        try {} catch (Exception e) {}
    }
}
