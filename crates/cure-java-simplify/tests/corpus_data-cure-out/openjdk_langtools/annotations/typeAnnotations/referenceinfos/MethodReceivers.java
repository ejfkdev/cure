import static java.lang.classfile.TypeAnnotation.TargetType.*;

public class MethodReceivers {
    @TADescription(annotation = "TA", type = METHOD_RECEIVER)
    public String regularMethod() {
        return "class %TEST_CLASS_NAME% { void test(@TA %TEST_CLASS_NAME% this) { } }";
    }
    @TADescription(annotation = "TA", type = METHOD_RECEIVER)
    public String abstractMethod() {
        return "abstract class %TEST_CLASS_NAME% { abstract void test(@TA %TEST_CLASS_NAME% this); }";
    }
    @TADescription(annotation = "TA", type = METHOD_RECEIVER)
    public String interfaceMethod() {
        return "interface %TEST_CLASS_NAME% { void test(@TA %TEST_CLASS_NAME% this); }";
    }
    @TADescription(annotation = "TA", type = METHOD_RECEIVER)
    public String regularWithThrows() {
        return "class %TEST_CLASS_NAME% { void test(@TA %TEST_CLASS_NAME% this) throws Exception { } }";
    }
    @TADescription(annotation = "TA", type = METHOD_RECEIVER,
            genericLocation = {})
    @TADescription(annotation = "TB", type = METHOD_RECEIVER,
            genericLocation = {1, 0})
    @TestClass("%TEST_CLASS_NAME%$TestInner")
    public String nestedtypes1() {
        return "class %TEST_CLASS_NAME% { class TestInner { void test(@TA %TEST_CLASS_NAME%. @TB TestInner this) { } } }";
    }
    @TADescription(annotation = "TA", type = METHOD_RECEIVER,
            genericLocation = {})
    @TestClass("%TEST_CLASS_NAME%$TestInner")
    public String nestedtypes2() {
        return "class %TEST_CLASS_NAME% { class TestInner { void test(@TA %TEST_CLASS_NAME%.TestInner this) { } } }";
    }
    @TADescription(annotation = "TB", type = METHOD_RECEIVER,
            genericLocation = {1, 0})
    @TestClass("%TEST_CLASS_NAME%$TestInner")
    public String nestedtypes3() {
        return "class %TEST_CLASS_NAME% { class TestInner { void test(%TEST_CLASS_NAME%. @TB TestInner this) { } } }";
    }
    @TADescription(annotation = "RTAs", type = METHOD_RECEIVER)
    public String regularMethodRepeatableAnnotation() {
        return "class %TEST_CLASS_NAME% { void test(@RTA @RTA %TEST_CLASS_NAME% this) { } }";
    }
    @TADescription(annotation = "RTAs", type = METHOD_RECEIVER)
    public String abstractMethodRepeatablaAnnotation() {
        return "abstract class %TEST_CLASS_NAME% { abstract void test(@RTA @RTA %TEST_CLASS_NAME% this); }";
    }
    @TADescription(annotation = "RTAs", type = METHOD_RECEIVER)
    public String interfaceMethodRepeatableAnnotation() {
        return "interface %TEST_CLASS_NAME% { void test(@RTA @RTA %TEST_CLASS_NAME% this); }";
    }
    @TADescription(annotation = "RTAs", type = METHOD_RECEIVER)
    public String regularWithThrowsRepeatableAnnotation() {
        return "class %TEST_CLASS_NAME% { void test(@RTA @RTA %TEST_CLASS_NAME% this) throws Exception { } }";
    }
    @TADescription(annotation = "RTAs", type = METHOD_RECEIVER,
            genericLocation = {})
    @TADescription(annotation = "RTBs", type = METHOD_RECEIVER,
            genericLocation = {1, 0})
    @TestClass("%TEST_CLASS_NAME%$TestInner")
    public String nestedtypesRepeatableAnnotation1() {
        return "class %TEST_CLASS_NAME% { class TestInner { void test(@RTA @RTA %TEST_CLASS_NAME%. @RTB @RTB TestInner this) { } } }";
    }
    @TADescription(annotation = "RTAs", type = METHOD_RECEIVER,
            genericLocation = {})
    @TestClass("%TEST_CLASS_NAME%$TestInner")
    public String nestedtypesRepeatableAnnotation2() {
        return "class %TEST_CLASS_NAME% { class TestInner { void test(@RTA @RTA %TEST_CLASS_NAME%.TestInner this) { } } }";
    }
    @TADescription(annotation = "RTBs", type = METHOD_RECEIVER,
            genericLocation = {1, 0})
    @TestClass("%TEST_CLASS_NAME%$TestInner")
    public String nestedtypesRepeatableAnnotation3() {
        return "class %TEST_CLASS_NAME% { class TestInner { void test(%TEST_CLASS_NAME%. @RTB @RTB TestInner this) { } } }";
    }
}
