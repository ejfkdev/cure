package com.puppycrawl.tools.checkstyle.checks.modifier.redundantmodifier;

public interface InputRedundantModifierNestedDef {
    public enum MyInnerEnum1 {
    }
    static enum MyInnerEnum2 {
    }
    public static enum MyInnerEnum3 {
    }
    static public enum MyInnerEnum4 {
    }
    interface MyInnerInterface {
        public strictfp class MyInnerClass {
        }
    }
    public static class testClass {
    }
    public abstract static @interface testAnnotatedInterface {
    }
}

abstract @interface testAnnotatedInterface {
    public static enum testEnum {
        // 2 violations above:
        // 'Redundant 'public' modifier.'
        // 'Redundant 'static' modifier.'
        }

        interface testInterface {
            public static interface nestedInterface {
            // 2 violations above:
            // 'Redundant 'public' modifier.'
            // 'Redundant 'static' modifier.'

                public static class nestedClass {
                // 2 violations above:
                // 'Redundant 'public' modifier.'
                // 'Redundant 'static' modifier.'
                }

                public static @interface nestedAnnInterface {
                // 2 violations above:
                // 'Redundant 'public' modifier.'
                // 'Redundant 'static' modifier.'
                }

                public static enum nestedEnum {
                // 2 violations above:
                // 'Redundant 'public' modifier.'
                // 'Redundant 'static' modifier.'
                }
            }
        }
}
