package com.puppycrawl.tools.checkstyle.checks.indentation.indentation;

public class InputIndentationChainedMethodCallWithLambda {
    interface Stage {
        Stage thenCompose(java.util.function.Function<Object, Stage> fn);
        void thenAccept(java.util.function.Consumer<Object> fn);
    }
    void method1(String topicName, boolean authoritative) {
        validateAsync(topicName, authoritative).thenCompose((__) -> getTopicReferenceAsync(topicName)).thenAccept((topic) -> {
            if (topic == null) {
                return;
            }
        });
    }
    void method2(String topicName, boolean authoritative) {
        validateAsync(topicName, authoritative).thenCompose((__) -> getTopicReferenceAsync(topicName)).thenAccept((topic) -> {
            if (topic == null) {
                return;
            }
        });
    }
    private Stage validateAsync(String topicName, boolean authoritative) {
        return null;
    }
    private Stage getTopicReferenceAsync(String topicName) {
        return null;
    }
}
