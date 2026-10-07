import java.lang.classfile.*;
import java.lang.classfile.attribute.*;
import java.io.*;
import java.util.*;
import java.nio.file.Files;
import java.nio.charset.*;
import java.io.File;
import java.io.IOException;
import java.lang.annotation.*;
import static java.lang.annotation.RetentionPolicy.*;
import static java.lang.annotation.ElementType.*;

public class TestAnonInnerClasses extends ClassfileTestHelper {
    int errors = 0;
    int checks = 0;
    int tc = 0, xtc = 180;
    File testSrc = new File(System.getProperty("test.src"));
    AttributeMapper<?>[] AnnoAttributes = {Attributes.runtimeVisibleTypeAnnotations(), Attributes.runtimeInvisibleTypeAnnotations(), Attributes.runtimeVisibleAnnotations(), Attributes.runtimeInvisibleAnnotations()};
    String srcTemplate = "testanoninner.template";
    Boolean As = false, Bs = true, Cs = false, Ds = false, TAs = false, TBs = false;
    Boolean[][] bRepeat = {{false, false, false, false, false, false}, {true, false, true, false, true, false}, {false, true, false, true, false, true}, {true, true, true, true, true, true}};
    List<String> failed = new LinkedList<>();
    public static void main(String[] args) throws Exception {
        new TestAnonInnerClasses().run();
    }
    void check(String testcase, int vtaX, int itaX, int vaX, int iaX, int vtaA, int itaA, int vaA, int iaA) {
        String descr = " checking " + testcase + " _TYPE_, expected: " + vtaX + ", " + itaX + ", " + vaX + ", " + iaX + "; actual: " + vtaA + ", " + itaA + ", " + vaA + ", " + iaA;
        String description = descr.replace("_TYPE_", "RuntimeVisibleTypeAnnotations");
        if (vtaX != vtaA) {
            errors++;
            failed.add(++checks + " " + testcase + ": (vtaX) " + vtaX + " != " + vtaA + " (vtaA)");
            println(checks + " FAIL: " + description);
        } else {
            println(++checks + " PASS: " + description);
        }
        description = descr.replace("_TYPE_", "RuntimeInvisibleTypeAnnotations");
        if (itaX != itaA) {
            errors++;
            failed.add(++checks + " " + testcase + ": (itaX) " + itaX + " != " + itaA + " (itaA)");
            println(checks + " FAIL: " + description);
        } else {
            println(++checks + " PASS: " + description);
        }
        description = descr.replace("_TYPE_", "RuntimeVisibleAnnotations");
        if (vaX != vaA) {
            errors++;
            failed.add(++checks + " " + testcase + ": (vaX) " + vaX + " != " + vaA + " (vaA)");
            println(checks + " FAIL: " + description);
        } else {
            println(++checks + " PASS: " + description);
        }
        description = descr.replace("_TYPE_", "RuntimeInvisibleAnnotations");
        if (iaX != iaA) {
            errors++;
            failed.add(++checks + " " + testcase + ": (iaX) " + iaX + " != " + iaA + " (iaA)");
            println(checks + " FAIL: " + description);
        } else {
            println(++checks + " PASS: " + description);
        }
        println("");
    }
    void report() {
        if (errors != 0) {
            System.err.println("Failed tests: " + errors + "\nfailed test cases:\n");
            for (String t : failed) 
                System.err.println("  " + t);
            throw new RuntimeException("FAIL: There were test failures.");
        } else 
            System.out.println("PASSED all tests.");
    }
    <T extends Attribute<T>> void test(AttributedElement m) {
        int vtaActual = 0;
        int itaActual = 0;
        int vaActual = 0;
        int iaActual = 0;
        int vtaExp = 0;
        int itaExp = 0;
        int vaExp = 0;
        int iaExp = 0;
        int index = 0;
        int index2 = 0;
        String memberName = null;
        String testcase = "undefined";
        String testClassName = null;
        Attribute<T> attr = null;
        Attribute<T> cattr = null;
        CodeAttribute CAttr = null;
        for (AttributeMapper<?> Anno : AnnoAttributes) {
            AttributeMapper<T> AnnoType = (AttributeMapper<T>) Anno;
            if (Objects.requireNonNull(m) instanceof ClassModel) {
                memberName = ((ClassModel) m).thisClass().name().stringValue();
                attr = m.findAttribute(AnnoType).orElse(null);
            } else {
                memberName = ((MethodModel) m).methodName().stringValue();
                attr = m.findAttribute(AnnoType).orElse(null);
                CAttr = m.findAttribute(Attributes.code()).orElse(null);
                if (CAttr != null) {
                    cattr = CAttr.findAttribute(AnnoType).orElse(null);
                }
            }
            if (attr != null) {
                switch (attr) {
                    case RuntimeVisibleTypeAnnotationsAttribute RVTAa -> vtaActual += RVTAa.annotations().size();
                    case RuntimeVisibleAnnotationsAttribute RVAa -> vaActual += RVAa.annotations().size();
                    case RuntimeInvisibleTypeAnnotationsAttribute RITAa -> itaActual += RITAa.annotations().size();
                    case RuntimeInvisibleAnnotationsAttribute RIAa -> iaActual += RIAa.annotations().size();
                    default -> throw new AssertionError();
                }
            }
            if (cattr != null) {
                switch (cattr) {
                    case RuntimeVisibleTypeAnnotationsAttribute RVTAa -> vtaActual += RVTAa.annotations().size();
                    case RuntimeVisibleAnnotationsAttribute RVAa -> vaActual += RVAa.annotations().size();
                    case RuntimeInvisibleTypeAnnotationsAttribute RITAa -> itaActual += RITAa.annotations().size();
                    case RuntimeInvisibleAnnotationsAttribute RIAa -> iaActual += RIAa.annotations().size();
                    default -> throw new AssertionError();
                }
            }
        }
        switch (memberName) {
            case "test":
                vtaExp = 4;
                itaExp = 4;
                vaExp = 0;
                iaExp = 0;
                tc++;
                break;
            case "mtest":
                vtaExp = 4;
                itaExp = 4;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "m1":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "m2":
                vtaExp = 4;
                itaExp = 4;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "m3":
                vtaExp = 10;
                itaExp = 10;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "tm":
                vtaExp = 6;
                itaExp = 6;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "i_m1":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "i_m2":
                vtaExp = 4;
                itaExp = 4;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "i_um":
                vtaExp = 6;
                itaExp = 6;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "l_m1":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "l_m2":
                vtaExp = 4;
                itaExp = 4;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "l_um":
                vtaExp = 6;
                itaExp = 6;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "mm_m1":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "mm_m2":
                vtaExp = 4;
                itaExp = 4;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "mm_m3":
                vtaExp = 10;
                itaExp = 10;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "mm_tm":
                vtaExp = 6;
                itaExp = 6;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "ia_m1":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "ia_m2":
                vtaExp = 4;
                itaExp = 4;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "ia_um":
                vtaExp = 6;
                itaExp = 6;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "data":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "odata1":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "pdata1":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "tdata":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "sa1":
                vtaExp = 6;
                itaExp = 6;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "i_odata1":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "i_pdata1":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "i_udata":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "i_sa1":
                vtaExp = 6;
                itaExp = 6;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "i_tdata":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "l_odata1":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "l_pdata1":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "l_udata":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "l_sa1":
                vtaExp = 6;
                itaExp = 6;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "l_tdata":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "mm_odata1":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "mm_pdata1":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "mm_sa1":
                vtaExp = 6;
                itaExp = 6;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "mm_tdata":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "ia_odata1":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "ia_pdata1":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "ia_udata":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "ia_sa1":
                vtaExp = 6;
                itaExp = 6;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "ia_tdata":
                vtaExp = 2;
                itaExp = 2;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "IA":
                vtaExp = 4;
                itaExp = 4;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            case "IN":
                vtaExp = 4;
                itaExp = 4;
                vaExp = 1;
                iaExp = 1;
                tc++;
                break;
            default:
                vtaExp = 0;
                itaExp = 0;
                vaExp = 0;
                iaExp = 0;
                break;
        }
        check(testcase, vtaExp, itaExp, vaExp, iaExp, vtaActual, itaActual, vaActual, iaActual);
    }
    public <T extends Attribute<T>> void run() {
        ClassModel cm = null;
        int testcount = 1;
        File testFile = null;
        for (Boolean[] bCombo : bRepeat) {
            As = bCombo[0];
            Bs = bCombo[1];
            Cs = bCombo[2];
            Ds = bCombo[3];
            TAs = bCombo[4];
            TBs = bCombo[5];
            String testname = "Test" + testcount++;
            println("Combinations: " + As + ", " + Bs + ", " + Cs + ", " + Ds + ", " + TAs + ", " + TBs + "; see " + testname + ".java");
            String[] classes = {testname + ".class", testname + "$Inner.class", testname + "$1Local1.class", testname + "$1.class", testname + "$1$1.class", testname + "$1$InnerAnon.class"};
            String sourceString = getSource(srcTemplate, testname, As, Bs, Cs, Ds, TAs, TBs);
            System.out.println(sourceString);
            try {
                testFile = writeTestFile(testname + ".java", sourceString);
            } catch (IOException ioe) {
                ioe.printStackTrace();
            }
            File classFile = null;
            try {
                classFile = compile(testFile);
            } catch (Error err) {
                System.err.println("FAILED compile. Source:\n" + sourceString);
                throw err;
            }
            String testloc = classFile.getAbsolutePath().substring(0, classFile.getAbsolutePath().indexOf(classFile.getPath()));
            for (String clazz : classes) {
                try {
                    cm = ClassFile.of().parse(new File(testloc + clazz).toPath());
                } catch (Exception e) {
                    e.printStackTrace();
                }
                assert cm != null;
                for (MethodModel m : cm.methods()) {
                    test(m);
                }
                for (FieldModel f : cm.fields()) {
                    test(f);
                }
            }
        }
        report();
        if (tc != xtc) 
            System.out.println("Test Count: " + tc + " != expected: " + xtc);
    }
    String getSrcTemplate(String sTemplate) {
        List<String> tmpl = null;
        String sTmpl = "";
        try {
            tmpl = Files.readAllLines(new File(testSrc, sTemplate).toPath(), Charset.defaultCharset());
        } catch (IOException ioe) {
            ioe.printStackTrace();
            throw new RuntimeException("FAILED: Test failed to read template" + sTemplate);
        }
        for (String l : tmpl) 
            sTmpl = sTmpl.concat(l).concat("\n");
        return sTmpl;
    }
    String getSource(String templateName, String testname, Boolean Arepeats, Boolean Brepeats, Boolean Crepeats, Boolean Drepeats, Boolean TArepeats, Boolean TBrepeats) {
        String testsource = getSrcTemplate(templateName).replace("testname", testname);
        testsource = testsource.replace("_As", Arepeats ? "@A @A" : "@A").replace("_Bs", Brepeats ? "@B @B" : "@B").replace("_Cs", Crepeats ? "@C @C" : "@C");
        testsource = testsource.replace("_Ds", Drepeats ? "@D @D" : "@D").replace("_TAs", TArepeats ? "@TA @TA" : "@TA").replace("_TBs", TBrepeats ? "@TB @TB" : "@TB");
        return testsource;
    }
}
