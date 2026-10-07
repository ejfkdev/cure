package com.puppycrawl.tools.checkstyle.checks.modifier.classmemberimpliedmodifier;

public class InputClassMemberImpliedModifierRecords {
    public static interface GoodInterface {
    }
    public interface BadInterface {
    }
    public static enum GoodEnum {
    }
    public enum BadEnum {
    }
    public static record GoodRecord() {
    }
    public record BadRecord() {
    }
    public static record OuterRecord() {
        public static record InnerRecord1() {
        }
        public record InnerRecord2() {
        }
        public static interface InnerInterface1 {
        }
        public interface InnerInterface2 {
        }
        public static enum InnerEnum1 {
        }
        public enum InnerEnum2 {
        }
    }
    Object obj = new Object() {
        // violation below 'Implied modifier 'static' should be explicit.'
        public record BadRecord() {}
        public static record OkRecord() {}
    };
}
