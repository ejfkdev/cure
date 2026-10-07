import static java.lang.classfile.TypeAnnotation.TargetType.*;

public class Fields {
    @TADescription(annotation = "TA", type = FIELD)
    public String fieldAsPrimitive() {
        return "@TA int test;";
    }
    @TADescription(annotation = "TA", type = FIELD)
    public String fieldAsObject() {
        return "@TA Object test;";
    }
    @TADescription(annotation = "TA", type = FIELD)
    @TADescription(annotation = "TB", type = FIELD,
            genericLocation = { 3, 0 })
    @TADescription(annotation = "TC", type = FIELD,
            genericLocation = { 3, 1 })
    @TADescription(annotation = "TD", type = FIELD,
            genericLocation = { 3, 1, 3, 0 })
    public String fieldAsParametrized() {
        return "@TA Map<@TB String, @TC List<@TD String>> test;";
    }
    @TADescription(annotation = "TA", type = FIELD)
    @TADescription(annotation = "TB", type = FIELD,
            genericLocation = { 0, 0 })
    @TADescription(annotation = "TC", type = FIELD,
            genericLocation = { 0, 0, 0, 0 })
    public String fieldAsArray() {
        return "@TC String @TA [] @TB [] test;";
    }
    @TADescription(annotation = "TA", type = FIELD)
    @TADescription(annotation = "TB", type = FIELD,
            genericLocation = { 0, 0 })
    @TADescription(annotation = "TC", type = FIELD,
            genericLocation = { 0, 0, 0, 0, 1, 0})
    public String fieldAsNestedArray1() {
        return "@TC Nested @TA [] @TB [] test;";
    }
    @TADescription(annotation = "TA", type = FIELD)
    @TADescription(annotation = "TB", type = FIELD,
            genericLocation = { 0, 0 })
    @TADescription(annotation = "TC", type = FIELD,
            genericLocation = { 0, 0, 0, 0, 1, 0})
    @TADescription(annotation = "TD", type = FIELD,
            genericLocation = { 0, 0, 0, 0 })
    public String fieldAsNestedArray2() {
        return "@TD %TEST_CLASS_NAME%. @TC Nested @TA [] @TB [] test;";
    }
    @TADescription(annotation = "TA", type = FIELD)
    @TADescription(annotation = "TB", type = FIELD,
            genericLocation = { 0, 0 })
    @TADescription(annotation = "TC", type = FIELD,
            genericLocation = { 0, 0, 0, 0 })
    public String fieldAsArrayOld() {
        return "@TC String test @TA [] @TB [];";
    }
    @TADescriptions({})
    public String fieldWithDeclarationAnnotatin() {
        return "@Decl String test;";
    }
    @TADescriptions({})
    public String fieldWithNoTargetAnno() {
        return "@A String test;";
    }
    @TADescription(annotation = "TA", type = FIELD)
    public String interfaceFieldAsObject() {
        return "interface %TEST_CLASS_NAME% { @TA String test = null; }";
    }
    @TADescription(annotation = "TA", type = FIELD)
    public String abstractFieldAsObject() {
        return "abstract class %TEST_CLASS_NAME% { @TA String test; }";
    }
    @TADescription(annotation = "TA", type = FIELD)
    @TADescription(annotation = "TB", type = FIELD,
            genericLocation = { 3, 0 })
    @TADescription(annotation = "TC", type = FIELD,
            genericLocation = { 3, 1 })
    @TADescription(annotation = "TD", type = FIELD,
            genericLocation = { 3, 1, 3, 0 })
    public String interfaceFieldAsParametrized() {
        return "interface %TEST_CLASS_NAME% { @TA Map<@TB String, @TC List<@TD String>> test = null; }";
    }
    @TADescription(annotation = "TA", type = FIELD)
    @TADescription(annotation = "TB", type = FIELD,
            genericLocation = { 3, 0 })
    @TADescription(annotation = "TC", type = FIELD,
            genericLocation = { 3, 1 })
    @TADescription(annotation = "TD", type = FIELD,
            genericLocation = { 3, 1, 3, 0 })
    public String staticFieldAsParametrized() {
        return "static @TA Map<@TB String, @TC List<@TD String>> test;";
    }
    @TADescription(annotation = "RTAs", type = FIELD)
    public String fieldAsPrimitiveRepeatableAnnotation() {
        return "@RTA @RTA int test;";
    }
    @TADescription(annotation = "RTAs", type = FIELD)
    public String fieldAsObjectRepeatableAnnotation() {
        return "@RTA @RTA Object test;";
    }
    @TADescription(annotation = "RTAs", type = FIELD)
    @TADescription(annotation = "RTBs", type = FIELD,
            genericLocation = { 3, 0 })
    @TADescription(annotation = "RTCs", type = FIELD,
            genericLocation = { 3, 1 })
    @TADescription(annotation = "RTDs", type = FIELD,
            genericLocation = { 3, 1, 3, 0 })
    public String fieldAsParametrizedRepeatableAnnotation() {
        return "@RTA @RTA Map<@RTB @RTB String, @RTC @RTC List<@RTD @RTD String>> test;";
    }
    @TADescription(annotation = "RTAs", type = FIELD)
    @TADescription(annotation = "RTBs", type = FIELD,
            genericLocation = { 0, 0 })
    @TADescription(annotation = "RTCs", type = FIELD,
            genericLocation = { 0, 0, 0, 0 })
    public String fieldAsArrayRepeatableAnnotation() {
        return "@RTC @RTC String @RTA @RTA [] @RTB @RTB [] test;";
    }
    @TADescription(annotation = "RTAs", type = FIELD)
    @TADescription(annotation = "RTBs", type = FIELD,
           genericLocation = { 0, 0 })
    @TADescription(annotation = "RTCs", type = FIELD,
            genericLocation = { 0, 0, 0, 0 })
    public String fieldAsArrayOldRepeatableAnnotation() {
        return "@RTC @RTC String test @RTA @RTA [] @RTB @RTB [];";
    }
    @TADescription(annotation = "RTAs", type = FIELD)
    public String interfaceFieldAsObjectRepeatableAnnotation() {
        return "interface %TEST_CLASS_NAME% { @RTA @RTA String test = null; }";
    }
    @TADescription(annotation = "RTAs", type = FIELD)
    public String abstractFieldAsObjectRepeatableAnnotation() {
        return "abstract class %TEST_CLASS_NAME% { @RTA @RTA String test; }";
    }
    @TADescription(annotation = "RTAs", type = FIELD)
    @TADescription(annotation = "RTBs", type = FIELD,
            genericLocation = { 3, 0 })
    @TADescription(annotation = "RTCs", type = FIELD,
            genericLocation = { 3, 1 })
    @TADescription(annotation = "RTDs", type = FIELD,
            genericLocation = { 3, 1, 3, 0 })
    public String interfaceFieldAsParametrizedRepeatableAnnotation() {
        return "interface %TEST_CLASS_NAME% { @RTA @RTA Map<@RTB @RTB String, @RTC @RTC List<@RTD @RTD String>> test = null; }";
    }
    @TADescription(annotation = "RTAs", type = FIELD)
    @TADescription(annotation = "RTBs", type = FIELD,
            genericLocation = { 3, 0 })
    @TADescription(annotation = "RTCs", type = FIELD,
            genericLocation = { 3, 1 })
    @TADescription(annotation = "RTDs", type = FIELD,
            genericLocation = { 3, 1, 3, 0 })
    public String staticFieldAsParametrizedRepeatableAnnotation() {
        return "static @RTA @RTA Map<@RTB @RTB String, @RTC @RTC List<@RTD @RTD String>> test;";
    }
}
