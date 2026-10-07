import static java.lang.classfile.TypeAnnotation.TargetType.*;

public class TypeCasts {
    @TADescription(annotation = "TA", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String returnObject() {
        return "Object returnObject() { return (@TA String)null; }";
    }
    @TADescription(annotation = "TA", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "TB", type = CAST,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "TC", type = CAST,
            genericLocation = { 0, 0, 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String returnObjectArray() {
        return "Object returnObjectArray() { return (@TC String @TA [] @TB [])null; }";
    }
    @TADescription(annotation = "TA", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "TB", type = CAST,
            genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String returnObjectGeneric() {
        return "Object returnObjectGeneric() { return (@TA List<@TB String>)null; }";
    }
    @TADescription(annotation = "TA", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String returnPrim() {
        return "Object returnPrim() { return (@TA int)0; }";
    }
    @TADescription(annotation = "TA", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "TB", type = CAST,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String returnPrimArray() {
        return "Object returnPrimArray() { return (@TB int @TA [])null; }";
    }
    @TADescription(annotation = "TA", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String initObject() {
        return "void initObject() { Object a =  (@TA String)null; }";
    }
    @TADescription(annotation = "TA", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "TB", type = CAST,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String initObjectArray() {
        return "void initObjectArray() { Object a = (@TB String @TA [])null; }";
    }
    @TADescription(annotation = "TA", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "TB", type = CAST,
            genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String initObjectGeneric() {
        return "void initObjectGeneric() { Object a = (@TA List<@TB String>)null; }";
    }
    @TADescription(annotation = "TA", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String initPrim() {
        return "void initPrim() { Object a =  (@TA int)0; }";
    }
    @TADescription(annotation = "TA", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "TB", type = CAST,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String initPrimArray() {
        return "void initPrimArray() { Object a = (@TB int @TA [])null; }";
    }
    @TADescription(annotation = "TA", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String eqtestObject() {
        return "void eqtestObject() { if (null == (@TA String)null); }";
    }
    @TADescription(annotation = "TA", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "TB", type = CAST,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String eqtestObjectArray() {
        return "void eqtestObjectArray() { if (null == (@TB String @TA [])null); }";
    }
    @TADescription(annotation = "TA", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "TB", type = CAST,
            genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String eqtestObjectGeneric() {
        return "void eqtestObjectGeneric() { if (null == (@TA List<@TB String >)null); }";
    }
    @TADescription(annotation = "TA", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    // compiler optimizes away compile time constants casts
    public String eqtestPrim() {
        return "void eqtestPrim(int a) { if (0 == (@TA int)a); }";
    }
    @TADescription(annotation = "TA", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "TB", type = CAST,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String eqtestPrimArray() {
        return "void eqtestPrimArray() { if (null == (@TB int @TA [])null); }";
    }
    @TADescription(annotation = "TA", type = CAST,
            offset = ReferenceInfoUtil.IGNORE_VALUE, typeIndex = 0)
    @TADescription(annotation = "TB", type = CAST,
            offset = ReferenceInfoUtil.IGNORE_VALUE, typeIndex = 1)
    @TADescription(annotation = "TC", type = CAST,
            offset = ReferenceInfoUtil.IGNORE_VALUE, typeIndex = 1,
            genericLocation = {3, 0})
    public String intersection1() {
        return "void intersection() { Object o = (@TA String & @TB Comparable<@TC String>) null; }";
    }
    @TADescription(annotation = "TA", type = CAST,
            offset = ReferenceInfoUtil.IGNORE_VALUE, typeIndex = 0)
    @TADescription(annotation = "TB", type = CAST,
            offset = ReferenceInfoUtil.IGNORE_VALUE, typeIndex = 1)
    @TADescription(annotation = "TC", type = CAST,
            offset = ReferenceInfoUtil.IGNORE_VALUE, typeIndex = 1,
            genericLocation = {3, 0})
    @TADescription(annotation = "TD", type = CAST,
            offset = ReferenceInfoUtil.IGNORE_VALUE, typeIndex = 2)
    public String intersection2() {
        return "void intersection() { Object o = (@TA String & @TB Comparable<@TC String> & @TD CharSequence) null; }";
    }
    @TADescription(annotation = "RTAs", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String returnObjectRepeatableAnnotation() {
        return "Object returnObject() { return (@RTA @RTA String)null; }";
    }
    @TADescription(annotation = "RTAs", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "RTBs", type = CAST,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "RTCs", type = CAST,
            genericLocation = { 0, 0, 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String returnObjectArrayRepeatableAnnotation() {
        return "Object returnObjectArray() { return (@RTC @RTC String @RTA @RTA [] @RTB @RTB [])null; }";
    }
    @TADescription(annotation = "RTAs", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "RTBs", type = CAST,
            genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String returnObjectGenericRepeatableAnnotation() {
        return "Object returnObjectGeneric() { return (@RTA @RTA List<@RTB @RTB String>)null; }";
    }
    @TADescription(annotation = "RTAs", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String returnPrimRepeatableAnnotation() {
        return "Object returnPrim() { return (@RTA @RTA int)0; }";
    }
    @TADescription(annotation = "RTAs", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "RTBs", type = CAST,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String returnPrimArrayRepeatableAnnotation() {
        return "Object returnPrimArray() { return (@RTB @RTB int @RTA @RTA [])null; }";
    }
    @TADescription(annotation = "RTAs", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String initObjectRepeatableAnnotation() {
        return "void initObject() { Object a =  (@RTA @RTA String)null; }";
    }
    @TADescription(annotation = "RTAs", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "RTBs", type = CAST,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String initObjectArrayRepeatableAnnotation() {
        return "void initObjectArray() { Object a = (@RTB @RTB String @RTA @RTA [])null; }";
    }
    @TADescription(annotation = "RTAs", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "RTBs", type = CAST,
            genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String initObjectGenericRepeatableAnnotation() {
        return "void initObjectGeneric() { Object a = (@RTA @RTA List<@RTB @RTB String>)null; }";
    }
    @TADescription(annotation = "RTAs", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String initPrimRepeatableAnnotation() {
        return "void initPrim() { Object a =  (@RTA @RTA int)0; }";
    }
    @TADescription(annotation = "RTAs", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "RTBs", type = CAST,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String initPrimArrayRepeatableAnnotation() {
        return "void initPrimArray() { Object a = (@RTB @RTB int @RTA @RTA [])null; }";
    }
    @TADescription(annotation = "RTAs", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String eqtestObjectRepeatableAnnotation() {
        return "void eqtestObject() { if (null == (@RTA @RTA String)null); }";
    }
    @TADescription(annotation = "RTAs", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "RTBs", type = CAST,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String eqtestObjectArrayRepeatableAnnotation() {
        return "void eqtestObjectArray() { if (null == (@RTB @RTB String @RTA @RTA [])null); }";
    }
    @TADescription(annotation = "RTAs", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "RTBs", type = CAST,
            genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String eqtestObjectGenericRepeatableAnnotation() {
        return "void eqtestObjectGeneric() { if (null == (@RTA @RTA List<@RTB @RTB String >)null); }";
    }
    @TADescription(annotation = "RTAs", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    // compiler optimizes away compile time constants casts
    public String eqtestPrimRepeatableAnnotation() {
        return "void eqtestPrim(int a) { if (0 == (@RTA @RTA int)a); }";
    }
    @TADescription(annotation = "RTAs", type = CAST, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    @TADescription(annotation = "RTBs", type = CAST,
            genericLocation = { 0, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE,
            typeIndex = 0)
    public String eqtestPrimArrayRepeatableAnnotation() {
        return "void eqtestPrimArray() { if (null == (@RTB @RTB int @RTA @RTA [])null); }";
    }
    @TADescription(annotation = "RTAs", type = CAST,
            offset = ReferenceInfoUtil.IGNORE_VALUE, typeIndex = 0)
    @TADescription(annotation = "RTBs", type = CAST,
            offset = ReferenceInfoUtil.IGNORE_VALUE, typeIndex = 1)
    @TADescription(annotation = "RTCs", type = CAST,
            offset = ReferenceInfoUtil.IGNORE_VALUE, typeIndex = 1,
            genericLocation = {3, 0})
    public String intersectionRepeatableAnnotation1() {
        return "void intersection() { Object o = (@RTA @RTA String & @RTB @RTB Comparable<@RTC @RTC String>) null; }";
    }
    @TADescription(annotation = "RTAs", type = CAST,
            offset = ReferenceInfoUtil.IGNORE_VALUE, typeIndex = 0)
    @TADescription(annotation = "RTBs", type = CAST,
            offset = ReferenceInfoUtil.IGNORE_VALUE, typeIndex = 1)
    @TADescription(annotation = "RTCs", type = CAST,
            offset = ReferenceInfoUtil.IGNORE_VALUE, typeIndex = 1,
            genericLocation = {3, 0})
    @TADescription(annotation = "RTDs", type = CAST,
            offset = ReferenceInfoUtil.IGNORE_VALUE, typeIndex = 2)
    public String intersectionRepeatableAnnotation2() {
        return "void intersection() { Object o = (@RTA @RTA String & @RTB @RTB Comparable<@RTC @RTC String> & @RTD @RTD CharSequence) null; }";
    }
}
