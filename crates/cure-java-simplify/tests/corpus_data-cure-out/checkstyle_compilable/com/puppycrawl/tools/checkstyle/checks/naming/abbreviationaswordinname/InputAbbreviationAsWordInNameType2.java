package com.puppycrawl.tools.checkstyle.checks.naming.abbreviationaswordinname;

abstract class InputAbbreviationAsWordInNameType2 {
}

abstract class NonAAAAbstractClassName2 {
}

abstract class FactoryWithHARDName2 {
}

abstract class AbstractCLASSName2 {
    abstract class NonAbstractInnerClass {
    }
}

abstract class ClassFactory12 {
    abstract class WellNamedFactory {
    }
}

class NonAbstractClass12 {
}

class AbstractClass12 {
}

class Class1Factory12 {
}

abstract class AbstractClassName32 {
    class AbstractINNERSClass {
    }
}

abstract class Class3Factory2 {
    class WellNamedFACTORY {
        public void systematicMETHODName() {
            int SYSTEMATICVariableName = 1;
        }
    }
}

interface Directions2 {
    int RIGHT = 1;
    int LEFT = 2;
    int UP = 3;
    int DOWN = 4;
}

interface BadNameForInterface2 {
    void interfaceMethod();
}

abstract class NonAAAAbstractClassName22 {
    public int serialNUMBER = 6;
    public final int s1erialNUMBER = 6;
    private static int s2erialNUMBER = 6;
    private static final int s3erialNUMBER = 6;
}

interface Interface12 {
    String VALUELONG = "value";
}

interface Interface22 {
    static String VALUELONG = "value";
}

interface Interface32 {
    final String VALUELONG = "value";
}

interface Interface42 {
    final static String VALUELONG = "value";
}

class FIleNameFormatException2 extends Exception {
    private static final long serialVersionUID = 1L;
    public FIleNameFormatException2(Exception e) {
        super(e);
    }
}
