package com.puppycrawl.tools.checkstyle.checks.modifier.redundantmodifier;

public class InputRedundantModifierStaticModifierInNestedEnum {
    static enum NestedEnumWithRedundantStatic {
    }
    enum CorrectNestedEnum {
        VAL;
        static enum NestedEnumWithRedundantStatic {
        }
    }
    interface NestedInterface {
        static enum NestedEnumWithRedundantStatic {
        }
    }
}
