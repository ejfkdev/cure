import java.util.Set;
import java.util.List;
import java.io.IOException;
import java.lang.annotation.ElementType;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.EnumSet;
import javax.tools.Diagnostic;
import javax.tools.DiagnosticCollector;
import javax.tools.JavaFileObject;
import static java.lang.annotation.ElementType.ANNOTATION_TYPE;
import static java.lang.annotation.ElementType.CONSTRUCTOR;
import static java.lang.annotation.ElementType.FIELD;
import static java.lang.annotation.ElementType.METHOD;
import static java.lang.annotation.ElementType.PARAMETER;
import static java.lang.annotation.ElementType.TYPE;
import static java.lang.annotation.ElementType.PACKAGE;
import static java.lang.annotation.ElementType.LOCAL_VARIABLE;
import static java.lang.annotation.ElementType.TYPE_USE;
import static java.lang.annotation.ElementType.TYPE_PARAMETER;
import static java.lang.annotation.ElementType.RECORD_COMPONENT;

public class TargetAnnoCombo {
    static final String TESTPKG = "testpkg";
    static final boolean DEBUG = false;
    final static Set<ElementType> noSet = null;
    final static Set<ElementType> empty = EnumSet.noneOf(ElementType.class);
    final static Set<ElementType> allTargets = EnumSet.allOf(ElementType.class);
    final static Set<ElementType> jdk7 = EnumSet.range(TYPE, PACKAGE);
    final static Set<ElementType> jdk8 = EnumSet.range(TYPE_PARAMETER, TYPE_USE);
    List<TestCase> testCases = new ArrayList<TestCase>();
    int errors = 0;
    enum IgnoreKind {
        RUN, IGNORE
    }
    private class TestCase {
        private Set<ElementType> baseAnnotations;
        private Set<ElementType> containerAnnotations;
        private IgnoreKind ignore;
        java.util.List<String> options;
        public TestCase(Set<ElementType> baseAnnotations, Set<ElementType> containerAnnotations) {
            this(baseAnnotations, containerAnnotations, IgnoreKind.RUN, null);
        }
        public TestCase(Set<ElementType> baseAnnotations, Set<ElementType> containerAnnotations, List<String> options) {
            this(baseAnnotations, containerAnnotations, IgnoreKind.RUN, options);
        }
        public TestCase(Set<ElementType> baseAnnotations, Set<ElementType> containerAnnotations, IgnoreKind ignoreKind, java.util.List<String> options) {
            this.baseAnnotations = baseAnnotations;
            this.containerAnnotations = containerAnnotations;
            this.ignore = ignoreKind;
            this.options = options;
        }
        public Set getBaseAnnotations() {
            return baseAnnotations;
        }
        public Set getContainerAnnotations() {
            return containerAnnotations;
        }
        public boolean isIgnored() {
            return ignore == IgnoreKind.IGNORE;
        }
        private boolean isValidSubSet() {
            if (baseAnnotations == null) {
                return containerAnnotations == null || !(containerAnnotations.contains(TYPE_USE) || containerAnnotations.contains(TYPE_PARAMETER));
            }
            Set<ElementType> tempBaseSet = EnumSet.noneOf(ElementType.class);
            tempBaseSet.addAll(baseAnnotations);
            if (baseAnnotations.contains(TYPE)) {
                tempBaseSet.add(ANNOTATION_TYPE);
            }
            if (baseAnnotations.contains(TYPE_USE)) {
                tempBaseSet.add(ANNOTATION_TYPE);
                tempBaseSet.add(TYPE);
                tempBaseSet.add(TYPE_PARAMETER);
            }
            return containerAnnotations == null ? tempBaseSet.containsAll(jdk7) : containerAnnotations.isEmpty() ? true : baseAnnotations.isEmpty() ? false : tempBaseSet.containsAll(containerAnnotations);
        }
    }
    public static void main(String[] args) throws Exception {
        TargetAnnoCombo tac = new TargetAnnoCombo();
        tac.generate();
        List<Integer> cases = new ArrayList<Integer>();
        for (int i = 0; i < args.length; i++) {
            cases.add(Integer.parseInt(args[i]));
        }
        if (cases.isEmpty()) {
            tac.run();
        } else {
            for (int index : cases) {
                tac.executeTestCase(tac.testCases.get(index), index);
            }
        }
    }
    List<String> source8 = List.of("-source", "8");
    private void generate() {
        testCases.addAll(Arrays.asList(new TestCase(noSet, noSet), new TestCase(noSet, empty), new TestCase(noSet, less(jdk8, TYPE_PARAMETER), source8), new TestCase(noSet, less(jdk8, TYPE_USE), source8), new TestCase(noSet, jdk8, source8), new TestCase(noSet, plus(EnumSet.range(TYPE, LOCAL_VARIABLE), TYPE_USE)), new TestCase(noSet, plus(EnumSet.range(TYPE, LOCAL_VARIABLE), TYPE_PARAMETER)), new TestCase(noSet, plus(empty, TYPE)), new TestCase(noSet, plus(empty, PARAMETER)), new TestCase(noSet, plus(empty, PACKAGE)), new TestCase(noSet, plus(empty, METHOD)), new TestCase(noSet, plus(empty, LOCAL_VARIABLE)), new TestCase(noSet, plus(empty, FIELD)), new TestCase(noSet, plus(empty, CONSTRUCTOR)), new TestCase(noSet, plus(empty, ANNOTATION_TYPE)), new TestCase(empty, noSet), new TestCase(empty, empty), new TestCase(empty, plus(empty, TYPE)), new TestCase(empty, plus(empty, PARAMETER)), new TestCase(empty, plus(empty, PACKAGE)), new TestCase(empty, plus(empty, METHOD)), new TestCase(empty, plus(empty, LOCAL_VARIABLE)), new TestCase(empty, plus(empty, FIELD)), new TestCase(empty, plus(empty, CONSTRUCTOR)), new TestCase(empty, plus(empty, ANNOTATION_TYPE)), new TestCase(empty, less(jdk8, TYPE_USE), source8), new TestCase(empty, less(jdk8, TYPE_PARAMETER), source8), new TestCase(less(jdk7, TYPE), noSet, source8), new TestCase(less(jdk7, PARAMETER), noSet, source8), new TestCase(less(jdk7, PACKAGE), noSet, source8), new TestCase(less(jdk7, METHOD), noSet, source8), new TestCase(less(jdk7, LOCAL_VARIABLE), noSet, source8), new TestCase(less(jdk7, FIELD), noSet, source8), new TestCase(less(jdk7, CONSTRUCTOR), noSet, source8), new TestCase(less(jdk7, ANNOTATION_TYPE), noSet, source8), new TestCase(less(jdk7, TYPE, ANNOTATION_TYPE), noSet), new TestCase(jdk7, noSet, source8), new TestCase(plus(jdk7, TYPE_USE), noSet, source8), new TestCase(plus(jdk7, TYPE_PARAMETER), noSet, source8), new TestCase(allTargets, noSet, null), new TestCase(plus(empty, TYPE), empty), new TestCase(plus(empty, PARAMETER), empty), new TestCase(plus(empty, PACKAGE), empty), new TestCase(plus(empty, METHOD), empty), new TestCase(plus(empty, LOCAL_VARIABLE), empty), new TestCase(plus(empty, FIELD), empty), new TestCase(plus(empty, CONSTRUCTOR), empty), new TestCase(plus(empty, ANNOTATION_TYPE), empty), new TestCase(plus(empty, TYPE_USE), empty), new TestCase(plus(empty, TYPE_PARAMETER), empty), new TestCase(allTargets, allTargets), new TestCase(allTargets, less(allTargets, TYPE)), new TestCase(allTargets, less(allTargets, PARAMETER)), new TestCase(allTargets, less(allTargets, PACKAGE)), new TestCase(allTargets, less(allTargets, METHOD)), new TestCase(allTargets, less(allTargets, LOCAL_VARIABLE)), new TestCase(allTargets, less(allTargets, FIELD)), new TestCase(allTargets, less(allTargets, CONSTRUCTOR)), new TestCase(allTargets, less(allTargets, ANNOTATION_TYPE)), new TestCase(allTargets, less(allTargets, TYPE_USE)), new TestCase(allTargets, less(allTargets, TYPE_PARAMETER)), new TestCase(less(allTargets, TYPE), allTargets), new TestCase(less(allTargets, PARAMETER), allTargets), new TestCase(less(allTargets, PACKAGE), allTargets), new TestCase(less(allTargets, METHOD), allTargets), new TestCase(less(allTargets, LOCAL_VARIABLE), allTargets), new TestCase(less(allTargets, FIELD), allTargets), new TestCase(less(allTargets, CONSTRUCTOR), allTargets), new TestCase(less(allTargets, ANNOTATION_TYPE), allTargets), new TestCase(less(allTargets, TYPE_USE), allTargets), new TestCase(less(allTargets, TYPE_PARAMETER), allTargets)));
        for (ElementType b : allTargets) {
            for (ElementType c : allTargets) {
                testCases.add(new TestCase(plus(empty, b), plus(empty, c)));
            }
        }
    }
    void run() throws Exception {
        int testCtr = 0;
        for (TestCase tc : testCases) {
            if (!tc.isIgnored()) {
                executeTestCase(tc, testCases.indexOf(tc));
                testCtr++;
            }
        }
        System.out.println("Total tests run: " + testCtr);
        if (errors > 0) {
            throw new Exception(errors + " errors found");
        }
    }
    private void executeTestCase(TestCase testCase, int index) {
        debugPrint("Test case number = " + index);
        debugPrint(" => baseAnnoTarget = " + testCase.getBaseAnnotations());
        debugPrint(" => containerAnnoTarget = " + testCase.getContainerAnnotations());
        String className = "TC" + index;
        boolean shouldCompile = testCase.isValidSubSet();
        Iterable<? extends JavaFileObject> files = getFileList(className, testCase, shouldCompile);
        boolean result = getCompileResult(className, shouldCompile, files, testCase.options);
        if (!result) {
            System.out.println("FAIL: Test " + index);
            try {
                for (JavaFileObject f : files) {
                    System.out.println("File: " + f.getName() + "\n" + f.getCharContent(true));
                }
            } catch (IOException ioe) {
                System.out.println("Exception: " + ioe);
            }
        } else {
            debugPrint("PASS: Test " + index);
        }
    }
    private Iterable<? extends JavaFileObject> getFileList(String className, TestCase testCase, boolean shouldCompile) {
        Set<ElementType> baseAnnoTarget = testCase.getBaseAnnotations();
        Set<ElementType> conAnnoTarget = testCase.getContainerAnnotations();
        String srcContent = "";
        String pkgInfoContent = "";
        String template = Helper.template;
        String baseTarget = "";
        String conTarget = "";
        String target = Helper.ContentVars.TARGET.getVal();
        if (baseAnnoTarget != null) {
            baseTarget = target.replace("#VAL", convertToString(baseAnnoTarget).toString()).replace("[", "{").replace("]", "}");
        }
        if (conAnnoTarget != null) {
            conTarget = target.replace("#VAL", convertToString(conAnnoTarget).toString()).replace("[", "{").replace("]", "}");
        }
        String annoData = Helper.ContentVars.IMPORTSTMTS.getVal() + conTarget + Helper.ContentVars.CONTAINER.getVal() + baseTarget + Helper.ContentVars.REPEATABLE.getVal() + Helper.ContentVars.BASE.getVal();
        JavaFileObject pkgInfoFile = null;
        if (shouldCompile && conAnnoTarget == null) {
            Set<ElementType> copySet = EnumSet.noneOf(ElementType.class);
            copySet.addAll(jdk7);
            conAnnoTarget = copySet;
        }
        if (shouldCompile) {
            boolean isPkgCasePresent = conAnnoTarget.contains(PACKAGE);
            String repeatableAnno = Helper.ContentVars.BASEANNO.getVal() + " " + Helper.ContentVars.BASEANNO.getVal();
            for (ElementType s : conAnnoTarget) {
                String replaceStr = "/*" + s.name() + "*/";
                if (s.name().equalsIgnoreCase("PACKAGE")) {
                    pkgInfoContent = repeatableAnno + "\npackage " + TESTPKG + ";" + annoData;
                    pkgInfoFile = Helper.getFile(TESTPKG + ".package-info", pkgInfoContent);
                } else {
                    template = template.replace(replaceStr, repeatableAnno);
                    if (!isPkgCasePresent) {
                        srcContent = template.replace("/*ANNODATA*/", annoData).replace("#ClassName", className);
                    } else {
                        replaceStr = "/*PACKAGE*/";
                        srcContent = template.replace(replaceStr, "package " + TESTPKG + ";").replace("#ClassName", className);
                    }
                }
            }
        } else {
            template = "class #ClassName {}";
            srcContent = annoData + template.replace("#ClassName", className);
        }
        JavaFileObject srcFile = Helper.getFile(className, srcContent);
        return pkgInfoFile != null ? Arrays.asList(pkgInfoFile, srcFile) : Arrays.asList(srcFile);
    }
    private boolean getCompileResult(String className, boolean shouldCompile, Iterable<? extends JavaFileObject> files, Iterable<String> options) {
        DiagnosticCollector<JavaFileObject> diagnostics = new DiagnosticCollector<JavaFileObject>();
        Helper.compileCode(diagnostics, files, options);
        boolean ok = false;
        String errMesg = "";
        int numDiags = diagnostics.getDiagnostics().size();
        if (numDiags == 0) {
            if (shouldCompile) {
                debugPrint("Test passed, compiled as expected.");
                ok = true;
            } else {
                errMesg = "Test failed, compiled unexpectedly.";
                ok = false;
            }
        } else {
            if (shouldCompile) {
                List<Diagnostic<? extends JavaFileObject>> allDiagnostics = diagnostics.getDiagnostics();
                if (allDiagnostics.stream().noneMatch((d) -> d.getKind() == javax.tools.Diagnostic.Kind.ERROR)) {
                    ok = true;
                } else {
                    errMesg = "Test failed, should have compiled successfully.";
                    ok = false;
                }
            } else {
                String expectedErrKey = "compiler.err.invalid.repeatable.annotation.incompatible.target";
                for (Diagnostic<?> d : diagnostics.getDiagnostics()) {
                    if (d.getKind() == Diagnostic.Kind.ERROR && d.getCode().contains(expectedErrKey)) {
                        debugPrint("Error message as expected.");
                        ok = true;
                        break;
                    } else {
                        ok = false;
                    }
                }
                if (!ok) {
                    errMesg = "Incorrect error received when compiling " + className + ", expected: " + expectedErrKey;
                }
            }
        }
        if (!ok) {
            error(errMesg);
            for (Diagnostic<?> d : diagnostics.getDiagnostics()) {
                System.out.println(" Diags: " + d);
            }
        }
        return ok;
    }
    private Set<ElementType> less(Set<ElementType> base, ElementType... sub) {
        Set<ElementType> res = EnumSet.noneOf(ElementType.class);
        res.addAll(base);
        for (ElementType t : sub) {
            res.remove(t);
        }
        return res;
    }
    private Set<ElementType> plus(Set<ElementType> base, ElementType... add) {
        Set<ElementType> res = EnumSet.noneOf(ElementType.class);
        res.addAll(base);
        for (ElementType t : add) {
            res.add(t);
        }
        return res;
    }
    private List<String> convertToString(Set<ElementType> annoTarget) {
        if (annoTarget == null) {
            return null;
        }
        List<String> annoTargets = new ArrayList<String>();
        for (ElementType e : annoTarget) {
            annoTargets.add("ElementType." + e.name());
        }
        return annoTargets;
    }
    private void debugPrint(String string) {
        if (DEBUG) {
            System.out.println(string);
        }
    }
    private void error(String msg) {
        System.out.println("ERROR: " + msg);
        errors++;
    }
}
