package com.puppycrawl.tools.checkstyle.checks.modifier.classmemberimpliedmodifier;

public class InputClassMemberImpliedModifierNoViolationRecords {
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
        public record BadRecord() {}
        public static record OkRecord() {}
    };
}
