import static java.lang.classfile.TypeAnnotation.TargetType.RESOURCE_VARIABLE;
import static java.lang.System.lineSeparator;

public class ResourceVariable {
    @TADescription(annotation = "TA", type = RESOURCE_VARIABLE,
            lvarOffset = {10}, lvarLength = {37}, lvarIndex = {1})
    @TADescription(annotation = "TB", type = RESOURCE_VARIABLE,
            lvarOffset = {20}, lvarLength = {4}, lvarIndex = {2})
    public String testResourceVariable() {
        return "public void f() throws IOException {" + lineSeparator() + "    try (@TA InputStream is1 = new FileInputStream(\"\")) {" + lineSeparator() + "        try (@TB InputStream is2 = new FileInputStream(\"\")) {}" + lineSeparator() + "    }" + lineSeparator() + "}";
    }
    @TADescription(annotation = "RTAs", type = RESOURCE_VARIABLE,
            lvarOffset = {10}, lvarLength = {4}, lvarIndex = {1})
    public String testRepeatedAnnotation1() {
        return "public void f() throws IOException {" + lineSeparator() + "    try (@RTA @RTA InputStream is1 = new FileInputStream(\"\")) {}" + lineSeparator() + "}";
    }
    @TADescription(annotation = "RTAs", type = RESOURCE_VARIABLE,
            lvarOffset = {10}, lvarLength = {4}, lvarIndex = {1})
    public String testRepeatedAnnotation2() {
        return "public void f() throws IOException {" + lineSeparator() + "    try (@RTAs({@RTA, @RTA}) InputStream is1 = new FileInputStream(\"\")) {}" + lineSeparator() + "}";
    }
    @TADescription(annotation = "TA", type = RESOURCE_VARIABLE,
            lvarOffset = {10}, lvarLength = {37}, lvarIndex = {1})
    @TADescription(annotation = "TB", type = RESOURCE_VARIABLE,
            lvarOffset = {20}, lvarLength = {4}, lvarIndex = {2})
    public String testSeveralVariablesInTryWithResources() {
        return "public void f() throws IOException {" + lineSeparator() + "    try (@TA InputStream is1 = new FileInputStream(\"\");" + lineSeparator() + "        @TB InputStream is2 = new FileInputStream(\"\")) {}" + lineSeparator() + "}";
    }
}
