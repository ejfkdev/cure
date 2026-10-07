import java.lang.classfile.*;
import java.lang.classfile.attribute.*;
import java.lang.annotation.*;
import java.io.*;
import java.util.List;
import java.lang.annotation.*;
import java.util.Objects;
import static java.lang.annotation.RetentionPolicy.*;
import static java.lang.annotation.ElementType.*;

public class TestNewCastArray {
    int errors = 0;
    List<String> failedTests = new java.util.LinkedList<>();
    String[] testclasses = {"Test1", "Test2a", "Test3a", "Test4a", "Test5a", "Test2b", "Test3b", "Test4b", "Test5b", "Test6a"};
    public static void main(String[] args) throws Exception {
        new TestNewCastArray().run();
    }
    void check(String testcase, int expected, int actual) {
        String res = testcase + ": (expected) " + expected + ", " + actual + " (actual): ";
        if (expected == actual) {
            res = res.concat("PASS");
        } else {
            errors++;
            res = res.concat("FAIL");
            failedTests.add(res);
        }
        System.out.println(res);
    }
    void report() {
        if (errors != 0) {
            System.err.println("Failed tests: " + errors + "\nfailed test cases:\n");
            for (String t : failedTests) 
                System.err.println("  " + t);
            throw new RuntimeException("FAIL: There were test failures.");
        } else 
            System.out.println("PASS");
    }
    <T extends Attribute<T>> void test(String clazz, AttributedElement m, AttributeMapper<T> name, Boolean codeattr) {
        int actual = 0;
        int expected = 0;
        int cexpected = 0;
        String memberName;
        Attribute<T> attr = null;
        CodeAttribute cAttr;
        String testcase;
        switch (m) {
            case MethodModel mm -> {
                memberName = mm.methodName().stringValue();
                if (codeattr) {
                    cAttr = mm.findAttribute(Attributes.code()).orElse(null);
                    if (cAttr != null) {
                        attr = cAttr.findAttribute(name).orElse(null);
                    }
                } else {
                    attr = mm.findAttribute(name).orElse(null);
                }
            }
            case FieldModel fm -> {
                memberName = fm.fieldName().stringValue();
                if (codeattr) {
                    cAttr = fm.findAttribute(Attributes.code()).orElse(null);
                    if (cAttr != null) {
                        attr = cAttr.findAttribute(name).orElse(null);
                    }
                } else {
                    attr = fm.findAttribute(name).orElse(null);
                }
            }
            default -> throw new AssertionError();
        }
        testcase = clazz + " , Local: " + codeattr + ": " + memberName + ", " + name;
        if (attr != null) {
            switch (attr) {
                case RuntimeVisibleTypeAnnotationsAttribute tAttr -> {
                    actual += tAttr.annotations().size();
                }
                case RuntimeInvisibleTypeAnnotationsAttribute tAttr -> {
                    actual += tAttr.annotations().size();
                }
                default -> throw new AssertionError();
            }
        }
        assert memberName != null;
        if (memberName.compareTo("<init>") == 0) 
            memberName = clazz + memberName;
        switch (memberName) {
            case "Test1<init>":
                expected = 0;
                break;
            case "testr22_22":
                expected = 4;
                break;
            case "testr11_11":
                expected = 4;
                break;
            case "testr12_21":
                expected = 4;
                break;
            case "testr20_02":
                expected = 2;
                break;
            case "Test2a<init>":
                cexpected = 0;
                break;
            case "test00_00_11_11":
                cexpected = 4;
                break;
            case "test21_12_21_12":
                cexpected = 8;
                break;
            case "test_new1":
                cexpected = 2;
                break;
            case "test_new2":
                cexpected = 2;
                break;
            case "test_cast1":
                cexpected = 2;
                break;
            case "test_cast2":
                cexpected = 2;
                break;
            case "Test2b<init>":
                cexpected = 0;
                break;
            case "test20_02_20_02":
                cexpected = 4;
                break;
            case "test22_22_22_22":
                cexpected = 8;
                break;
            case "test_new3":
                cexpected = 1;
                break;
            case "test_new4":
                cexpected = 1;
                break;
            case "test_new5":
                cexpected = 2;
                break;
            case "test_cast3":
                cexpected = 1;
                break;
            case "test_cast4":
                cexpected = 2;
                break;
            case "Test3a<init>":
                cexpected = 10;
                break;
            case "SA_21_12c":
                cexpected = 0;
                break;
            case "SA_01_10c":
                expected = 0;
                break;
            case "SA_11_11c":
                expected = 0;
                break;
            case "Test3b<init>":
                cexpected = 6;
                break;
            case "SA_22_22c":
                cexpected = 0;
                break;
            case "SA_20_02c":
                cexpected = 0;
                break;
            case "Test3c<init>":
                cexpected = 8;
                break;
            case "SA_10_10":
                cexpected = 0;
                break;
            case "SA_10_01":
                cexpected = 0;
                break;
            case "SA_21_12":
                cexpected = 0;
                break;
            case "Test3d<init>":
                cexpected = 6;
                break;
            case "SA_20_02":
                cexpected = 0;
                break;
            case "SA_22_22":
                cexpected = 0;
                break;
            case "Test4a<init>":
                cexpected = 4;
                break;
            case "nS_21":
                cexpected = 0;
                break;
            case "nS_12":
                cexpected = 0;
                break;
            case "Test4b<init>":
                cexpected = 4;
                break;
            case "nS20":
                cexpected = 0;
                break;
            case "nS02":
                cexpected = 0;
                break;
            case "nS22":
                cexpected = 0;
                break;
            case "Test5a<init>":
                cexpected = 4;
                break;
            case "ci11":
                expected = 0;
                break;
            case "ci21":
                expected = 0;
                break;
            case "Test5b<init>":
                cexpected = 3;
                break;
            case "ci2":
                expected = 0;
                break;
            case "ci22":
                expected = 0;
                break;
            case "Test6a<init>":
                cexpected = 4;
                break;
            case "test6aPrimitiveArray":
                expected = 0;
                break;
            case "test6aRefArray":
                expected = 0;
                break;
            case "test6aMethod":
                cexpected = 4;
                break;
            default:
                expected = 0;
                break;
        }
        if (codeattr) 
            check(testcase, cexpected, actual); else 
            check(testcase, expected, actual);
    }
    public void run() {
        ClassModel cm = null;
        InputStream in;
        for (String clazz : testclasses) {
            String testclazz = "TestNewCastArray$" + clazz + ".class";
            System.out.println("Testing " + testclazz);
            try {
                in = Objects.requireNonNull(getClass().getResource(testclazz)).openStream();
                cm = ClassFile.of().parse(in.readAllBytes());
                in.close();
            } catch (Exception e) {
                e.printStackTrace();
            }
            assert cm != null;
            if (clazz.startsWith("Test1")) {
                for (FieldModel fm : cm.fields()) 
                    test(clazz, fm, Attributes.runtimeVisibleTypeAnnotations(), false);
                for (MethodModel mm : cm.methods()) 
                    test(clazz, mm, Attributes.runtimeVisibleTypeAnnotations(), false);
            } else {
                for (FieldModel fm : cm.fields()) 
                    test(clazz, fm, Attributes.runtimeVisibleTypeAnnotations(), true);
                for (MethodModel mm : cm.methods()) 
                    test(clazz, mm, Attributes.runtimeVisibleTypeAnnotations(), true);
            }
        }
        report();
    }
    static class Test1 {
        Test1() {}
        String[][] testr22_22(Test1 this, String param, String... vararg) {
            return new String[2][2];
        }
        String[][] testr11_11(Test1 this, String param, String... vararg) {
            return new String[2][2];
        }
        String[][] testr12_21(Test1 this, String param, String... vararg) {
            return new String[2][2];
        }
        String[][] testr20_02(Test1 this, String param, String... vararg) {
            return new String[2][2];
        }
    }
    static class Test2a {
        Test2a() {}
        Object o = new Integer(1);
        String[][] test00_00_11_11(Test2a this, String param, String... vararg) {
            String [] [] sarray = new String @A @B[2] @A @B [2];
            return sarray;
        }
        String[][] test21_12_21_12(Test2a this, String param, String... vararg) {
            String @A @A @B [] @A @B @B [] sarray = new String @A @A @B[2] @A @B @B [2];
            return sarray;
        }
        void test_new1() {
            String nS_21 = new @A @A @B String("Hello");
        }
        void test_new2() {
            String nS_12 = new @A @B @B String("Hello");
        }
        void test_cast1() {
            String tcs11 = (String) o;
        }
        void test_cast2() {
            String tcs21 = (String) o;
        }
    }
    static class Test2b {
        Test2b() {}
        Object o = new Integer(1);
        String[][] test20_02_20_02(Test2b this, String param, String... vararg) {
            String @A @A [] @B @B [] sarray = new String @A @A[2] @B @B [2];
            return sarray;
        }
        String[][] test22_22_22_22(Test2b this, String param, String... vararg) {
            String @A @A @B @B [] @A @A @B @B [] sarray = new String @A @A @B @B [2] @A @A @B @B [2];
            return sarray;
        }
        void test_new3() {
            String nS20 = new @A @A String("Hello");
        }
        void test_new4() {
            String nS02 = new @B @B String("Hello");
        }
        void test_new5() {
            String nS22 = new @A @A @B @B String("Hello");
        }
        void test_cast3() {
            String tcs2 = (String) o;
        }
        void test_cast4() {
            String tcs22 = (String) o;
        }
    }
    static class Test3a {
        Test3a() {}
        String[][] SA_21_12c;
        [2] @A @B @B[2];
        String[][] SA_01_10c;
        [2] @A [2];
        String[][] SA_11_11c;
        [2] @A @B [2];
    }
    static class Test3b {
        Test3b() {}
        String[][] SA_22_22c;
        [2] @A @A @B @B[2];
        String[][] SA_20_02c;
        [2] @B @B[2];
    }
    static class Test3c {
        Test3c() {}
        String[][] SA_10_10 = new String[2][2];
        String[][] SA_10_01 = new String[2][2];
        String[][] SA_21_12 = new String[2][2];
    }
    static class Test3d {
        Test3d() {}
        String[][] SA_20_02 = new String[2][2];
        String[][] SA_22_22 = new String[2][2];
    }
    static class Test4a {
        Test4a() {}
        String nS_21;
        ("Hello");
        String nS_12;
        ("Hello");
    }
    static class Test4b {
        Test4b() {}
        String nS20;
        ("Hello");
        String nS02;
        ("Hello");
        String nS22;
        ("Hello");
    }
    static class Test5a {
        Test5a() {}
        Object o = 1;
        Integer ci11 = (Integer) o;
        Integer ci21 = (Integer) o;
    }
    static class Test5b {
        Test5b() {}
        Object o = 1;
        Integer ci2 = (Integer) o;
        Integer ci22 = (Integer) o;
    }
    static class Test6a {
        Test6a() {}
        long l = 0;
        int[] test6aPrimitiveArray = new int[(int) l];
        Integer[] test6aRefArray = new Integer[(int) l];
        private void test6aMethod() {
            int[] primitiveArray = new int[(int) l];
            Integer[] refArray = new Integer[(int) l];
        }
    }
    @Retention(RUNTIME) @Target({TYPE_USE}) @Repeatable( AC.class ) @interface A {
    }
    @Retention(RUNTIME) @Target({TYPE_USE}) @Repeatable( BC.class ) @interface B {
    }
    @Retention(RUNTIME) @Target({FIELD}) @Repeatable( FC.class ) @interface F {
    }
    @Retention(RUNTIME) @Target({TYPE_USE}) @interface AC {
        A[] value();
    }
    @Retention(RUNTIME) @Target({TYPE_USE}) @interface BC {
        B[] value();
    }
    @Retention(RUNTIME) @Target({FIELD}) @interface FC {
        F[] value();
    }
}
