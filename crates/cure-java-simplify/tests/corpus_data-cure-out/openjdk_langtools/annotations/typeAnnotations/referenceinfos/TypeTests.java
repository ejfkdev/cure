import static java.lang.classfile.TypeAnnotation.TargetType.*;

public class TypeTests {
    @TADescription(annotation = "TA", type = INSTANCEOF, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String returnObject() {
        return "Object returnObject() { return null instanceof @TA String; }";
    }
    @TADescription(annotation = "TA", type = INSTANCEOF, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TB", type = INSTANCEOF,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TC", type = INSTANCEOF,
            genericLocation = { 0, 0, 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String returnObjectArray() {
        return "Object returnObjectArray() { return null instanceof @TC String @TA [] @TB []; }";
    }
    @TADescription(annotation = "TA", type = INSTANCEOF, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TB", type = INSTANCEOF,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TC", type = INSTANCEOF,
            genericLocation = { 0, 0, 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String returnPrimArray() {
        return "Object returnPrimArray() { return null instanceof @TC int @TA [] @TB []; }";
    }
    @TADescription(annotation = "TA", type = INSTANCEOF, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String initObject() {
        return "void initObject() { Object a =  null instanceof @TA String; }";
    }
    @TADescription(annotation = "TA", type = INSTANCEOF, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TB", type = INSTANCEOF,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TC", type = INSTANCEOF,
            genericLocation = { 0, 0, 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String initObjectArray() {
        return "void initObjectArray() { Object a = null instanceof @TC String @TA [] @TB []; }";
    }
    @TADescription(annotation = "TA", type = INSTANCEOF, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TB", type = INSTANCEOF,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TC", type = INSTANCEOF,
            genericLocation = { 0, 0, 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String initPrimArray() {
        return "void initPrimArray() { Object a = null instanceof @TC int @TA [] @TB []; }";
    }
    @TADescription(annotation = "TA", type = INSTANCEOF, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String eqtestObject() {
        return "void eqtestObject() { if (true == (null instanceof @TA String)); }";
    }
    @TADescription(annotation = "TA", type = INSTANCEOF, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TB", type = INSTANCEOF,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TC", type = INSTANCEOF,
            genericLocation = { 0, 0, 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String eqtestObjectArray() {
        return "void eqtestObjectArray() { if (true == (null instanceof @TC String @TA [] @TB [])); }";
    }
    @TADescription(annotation = "TA", type = INSTANCEOF, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TB", type = INSTANCEOF,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TC", type = INSTANCEOF,
            genericLocation = { 0, 0, 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String eqtestPrimArray() {
        return "void eqtestPrimArray() { if (true == (null instanceof @TC int @TA [] @TB [])); }";
    }
    @TADescription(annotation = "RTAs", type = INSTANCEOF, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String returnObjectRepeatableAnnotation() {
        return "Object returnObject() { return null instanceof @RTA @RTA String; }";
    }
    @TADescription(annotation = "RTAs", type = INSTANCEOF, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTBs", type = INSTANCEOF,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTCs", type = INSTANCEOF,
            genericLocation = { 0, 0, 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String returnObjectArrayRepeatableAnnotation() {
        return "Object returnObjectArray() { return null instanceof @RTC @RTC String @RTA @RTA [] @RTB @RTB []; }";
    }
    @TADescription(annotation = "RTAs", type = INSTANCEOF, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTBs", type = INSTANCEOF,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTCs", type = INSTANCEOF,
            genericLocation = { 0, 0, 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String returnPrimArrayRepeatableAnnotation() {
        return "Object returnPrimArrayRepeatableAnnotation() { return null instanceof @RTC @RTC int @RTA @RTA [] @RTB @RTB []; }";
    }
    @TADescription(annotation = "RTAs", type = INSTANCEOF, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String initObjectRepeatableAnnotation() {
        return "void initObject() { Object a =  null instanceof @RTA @RTA String; }";
    }
    @TADescription(annotation = "RTAs", type = INSTANCEOF, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTBs", type = INSTANCEOF,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTCs", type = INSTANCEOF,
            genericLocation = { 0, 0, 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String initObjectArrayRepeatableAnnotation() {
        return "void initObjectArray() { Object a = null instanceof @RTC @RTC String @RTA @RTA [] @RTB @RTB []; }";
    }
    @TADescription(annotation = "RTAs", type = INSTANCEOF, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTBs", type = INSTANCEOF,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTCs", type = INSTANCEOF,
            genericLocation = { 0, 0, 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String initPrimArrayRepeatableAnnotation() {
        return "void initPrimArray() { Object a = null instanceof @RTC @RTC int @RTA @RTA [] @RTB @RTB []; }";
    }
    @TADescription(annotation = "RTAs", type = INSTANCEOF, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String eqtestObjectRepeatableAnnotation() {
        return "void eqtestObject() { if (true == (null instanceof @RTA @RTA String)); }";
    }
    @TADescription(annotation = "RTAs", type = INSTANCEOF, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTBs", type = INSTANCEOF,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTCs", type = INSTANCEOF,
            genericLocation = { 0, 0, 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String eqtestObjectArrayRepeatableAnnotation() {
        return "void eqtestObjectArray() { if (true == (null instanceof @RTC @RTC String @RTA @RTA [] @RTB @RTB [])); }";
    }
    @TADescription(annotation = "RTAs", type = INSTANCEOF, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTBs", type = INSTANCEOF,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTCs", type = INSTANCEOF,
            genericLocation = { 0, 0, 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String eqtestPrimArrayRepeatableAnnotation() {
        return "void eqtestPrimArray() { if (true == (null instanceof @RTC @RTC int @RTA @RTA [] @RTB @RTB [])); }";
    }
}
