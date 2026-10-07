import static java.lang.classfile.TypeAnnotation.TargetType.*;

public class Initializers {
    @TADescription(annotation = "TA", type = NEW, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TB", type = NEW,
                genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
        public String instanceInit1() {
        return "class %TEST_CLASS_NAME% { { Object o = new @TA ArrayList<@TB String>(); } }";
    }
    @TADescription(annotation = "TA", type = NEW, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TB", type = NEW,
                genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TC", type = NEW, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TD", type = NEW,
                genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
        public String instanceInit2() {
        return "class %TEST_CLASS_NAME% { Object f = new @TA ArrayList<@TB String>();  { Object o = new @TC ArrayList<@TD String>(); } }";
    }
    @TADescription(annotation = "TA", type = NEW, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TB", type = NEW,
                genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
        public String staticInit1() {
        return "class %TEST_CLASS_NAME% { static { Object o = new @TA ArrayList<@TB String>(); } }";
    }
    @TADescription(annotation = "TA", type = NEW, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TB", type = NEW,
                genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TC", type = NEW, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TD", type = NEW,
                genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TE", type = NEW, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "TF", type = NEW,
                genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
        public String staticInit2() {
        return "class %TEST_CLASS_NAME% { Object f = new @TA ArrayList<@TB String>();  static Object g = new @TC ArrayList<@TD String>();  static { Object o = new @TE ArrayList<@TF String>(); } }";
    }
    @TADescription(annotation = "TA", type = CAST,
                typeIndex = 0, offset = ReferenceInfoUtil.IGNORE_VALUE)
        public String lazyConstantCast1() {
        return "class %TEST_CLASS_NAME% { public static final Object o = (@TA Object) null; }";
    }
    @TADescription(annotation = "RTAs", type = NEW, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTBs", type = NEW,
            genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String instanceInitRepeatableAnnotation1() {
        return "class %TEST_CLASS_NAME% { { Object o = new @RTA @RTA ArrayList<@RTB @RTB String>(); } }";
    }
    @TADescription(annotation = "RTAs", type = NEW, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTBs", type = NEW,
            genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTCs", type = NEW, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTDs", type = NEW,
            genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String instanceInitRepeatableAnnotation2() {
        return "class %TEST_CLASS_NAME% { Object f = new @RTA @RTA ArrayList<@RTB @RTB String>();  { Object o = new @RTC @RTC ArrayList<@RTD @RTD String>(); } }";
    }
    @TADescription(annotation = "RTAs", type = NEW, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTBs", type = NEW,
            genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String staticInitRepeatableAnnotation1() {
        return "class %TEST_CLASS_NAME% { static { Object o = new @RTA @RTA ArrayList<@RTB @RTB String>(); } }";
    }
    @TADescription(annotation = "RTAs", type = NEW, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTBs", type = NEW,
            genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTCs", type = NEW, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTDs", type = NEW,
            genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTEs", type = NEW, offset = ReferenceInfoUtil.IGNORE_VALUE)
    @TADescription(annotation = "RTFs", type = NEW,
            genericLocation = { 3, 0 }, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String staticInitRepeatableAnnotation2() {
        return "class %TEST_CLASS_NAME% { Object f = new @RTA @RTA ArrayList<@RTB @RTB String>();  static Object g = new @RTC @RTC ArrayList<@RTD @RTD String>();  static { Object o = new @RTE @RTE ArrayList<@RTF @RTF String>(); } }";
    }
    @TADescription(annotation = "RTAs", type = CAST,
            typeIndex = 0, offset = ReferenceInfoUtil.IGNORE_VALUE)
    public String lazyConstantCastRepeatableAnnotation1() {
        return "class %TEST_CLASS_NAME% { public static final Object o = (@RTA @RTA Object) null; }";
    }
}
