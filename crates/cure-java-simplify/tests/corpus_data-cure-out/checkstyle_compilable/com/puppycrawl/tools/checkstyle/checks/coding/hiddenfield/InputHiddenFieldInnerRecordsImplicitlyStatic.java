package com.puppycrawl.tools.checkstyle.checks.coding.hiddenfield;

import java.time.Clock;
import java.time.Instant;

public class InputHiddenFieldInnerRecordsImplicitlyStatic {
}

class Scratch {
    private final Clock clock;
    Scratch(Clock clock) {
        this.clock = clock;
    }
    public record State(String token, Instant expiresAt) {
        static int pointer = 0;
        public boolean isFresh(final Clock clock) {
            return Instant.now(clock).isBefore(expiresAt);
        }
        public int anInt(int pointer) {
            return pointer;
        }
    }
}

class TestOne {
    String name;
    record data(String str, int integer) {
        void method() {}
        public boolean isTrue(String name) {
            return true;
        }
    }
}
