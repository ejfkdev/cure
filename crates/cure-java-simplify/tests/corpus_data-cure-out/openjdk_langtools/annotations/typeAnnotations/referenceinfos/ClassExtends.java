import static java.lang.classfile.TypeAnnotation.TargetType.*;

public class ClassExtends {
    @TADescription(annotation = "TA", type = CLASS_EXTENDS, typeIndex = 65535)
    @TADescription(annotation = "TB", type = CLASS_EXTENDS, typeIndex = 1)
    public String regularClass() {
        return "class %TEST_CLASS_NAME% extends @TA Object implements Cloneable, @TB Runnable {  public void run() { } }";
    }
    @TADescription(annotation = "RTAs", type = CLASS_EXTENDS, typeIndex = 65535)
    @TADescription(annotation = "RTBs", type = CLASS_EXTENDS, typeIndex = 1)
    public String regularClassRepeatableAnnotation() {
        return "class %TEST_CLASS_NAME% extends @RTA @RTA Object implements Cloneable, @RTB @RTB Runnable {  public void run() { } }";
    }
    @TADescription(annotation = "TA", type = CLASS_EXTENDS, typeIndex = 65535,
            genericLocation = { 3, 0 })
    @TADescription(annotation = "TB", type = CLASS_EXTENDS, typeIndex = 1,
            genericLocation  = { 3, 1 })
    public String regularClassExtendsParametrized() {
        return "class %TEST_CLASS_NAME% extends HashMap<@TA String, String> implements Cloneable, Map<String, @TB String>{ } ";
    }
    @TADescription(annotation = "RTAs", type = CLASS_EXTENDS, typeIndex = 65535,
            genericLocation = { 3, 0 })
    @TADescription(annotation = "RTBs", type = CLASS_EXTENDS, typeIndex = 1,
            genericLocation  = { 3, 1 })
    public String regularClassExtendsParametrizedRepeatableAnnotation() {
        return "class %TEST_CLASS_NAME% extends HashMap<@RTA @RTA String, String> implements Cloneable, Map<String, @RTB @RTB String>{ } ";
    }
    @TADescription(annotation = "TA", type = CLASS_EXTENDS, typeIndex = 65535)
    @TADescription(annotation = "TB", type = CLASS_EXTENDS, typeIndex = 1)
    public String abstractClass() {
        return "abstract class %TEST_CLASS_NAME% extends @TA Date implements Cloneable, @TB Runnable {  public void run() { } }";
    }
    @TADescription(annotation = "RTAs", type = CLASS_EXTENDS, typeIndex = 65535)
    @TADescription(annotation = "RTBs", type = CLASS_EXTENDS, typeIndex = 1)
    public String abstractClassRepeatableAnnotation() {
        return "abstract class %TEST_CLASS_NAME% extends @RTA @RTA Date implements Cloneable, @RTB @RTB Runnable {  public void run() { } }";
    }
    @TADescription(annotation = "RTAs", type = CLASS_EXTENDS, typeIndex = 65535,
            genericLocation = { 3, 0 })
    @TADescription(annotation = "RTBs", type = CLASS_EXTENDS, typeIndex = 1,
            genericLocation  = { 3, 1 })
    public String abstractClassExtendsParametrized() {
        return "abstract class %TEST_CLASS_NAME% extends HashMap<@RTA @RTA String, String> implements Cloneable, Map<String, @RTB @RTB String>{ } ";
    }
    @TADescription(annotation = "TB", type = CLASS_EXTENDS, typeIndex = 1)
    public String regularInterface() {
        return "interface %TEST_CLASS_NAME% extends Cloneable, @TB Runnable { }";
    }
    @TADescription(annotation = "RTAs", type = CLASS_EXTENDS, typeIndex = 1)
    public String regularInterfaceRepetableAnnotation() {
        return "interface %TEST_CLASS_NAME% extends Cloneable, @RTA @RTA Runnable { }";
    }
    @TADescription(annotation = "TB", type = CLASS_EXTENDS, typeIndex = 1,
            genericLocation  = { 3, 1 })
    public String regularInterfaceExtendsParametrized() {
        return "interface %TEST_CLASS_NAME% extends Cloneable, Map<String, @TB String>{ } ";
    }
    @TADescription(annotation = "RTBs", type = CLASS_EXTENDS, typeIndex = 1,
            genericLocation  = { 3, 1 })
    public String regularInterfaceExtendsParametrizedRepeatableAnnotation() {
        return "interface %TEST_CLASS_NAME% extends Cloneable, Map<String, @RTB @RTB String>{ } ";
    }
    @TADescription(annotation = "TB", type = CLASS_EXTENDS, typeIndex = 1)
    public String regularEnum() {
        return "enum %TEST_CLASS_NAME% implements Cloneable, @TB Runnable { TEST; public void run() { } }";
    }
    @TADescription(annotation = "RTBs", type = CLASS_EXTENDS, typeIndex = 1)
    public String regularEnumRepeatableAnnotation() {
        return "enum %TEST_CLASS_NAME% implements Cloneable, @RTB @RTB Runnable { TEST; public void run() { } }";
    }
    @TADescription(annotation = "TB", type = CLASS_EXTENDS, typeIndex = 1,
            genericLocation  = { 3, 0 })
    public String regularEnumExtendsParametrized() {
        return "enum %TEST_CLASS_NAME% implements Cloneable, Comparator<@TB String> { TEST;  public int compare(String a, String b) { return 0; }}";
    }
    @TADescription(annotation = "RTBs", type = CLASS_EXTENDS, typeIndex = 1,
            genericLocation  = { 3, 0 })
    public String regularEnumExtendsParametrizedRepeatableAnnotation() {
        return "enum %TEST_CLASS_NAME% implements Cloneable, Comparator<@RTB @RTB String> { TEST;  public int compare(String a, String b) { return 0; }}";
    }
}
