package com.puppycrawl.tools.checkstyle.checks.naming.abstractclassname;

abstract public class InputAbstractClassNameFormerFalsePositive {
}

abstract class AbstractClassFP {
}

abstract class AbstractClassOtherFP {
    abstract class NonAbstractInnerClassFP {
    }
}

class NonAbstractClassFP {
}

class AbstractClassNameFP {
}

abstract class AbstractClassName2FP {
    class AbstractInnerClassFP {
    }
}
