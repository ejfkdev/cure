import java.io.IOException;
import tools.javac.combo.*;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;

class BridgeMethodsTemplateTest extends BridgeMethodTestCase {
    @Test void test1() throws IOException, ReflectiveOperationException {
        compileSpec("Cc1(A)");
        assertLinkage("C", LINKAGE_ERROR, "C1");
        recompileSpec("Cc1(Ac0)", "A");
        assertLinkage("C", "A0", "C1");
    }
    @Test void test2() throws IOException, ReflectiveOperationException {
        compileSpec("Cc1(I)");
        assertLinkage("C", LINKAGE_ERROR, "C1");
        recompileSpec("Cc1(Id0)", "I");
        assertLinkage("C", "I0", "C1");
    }
    @Test void test3() throws IOException, ReflectiveOperationException {
        compileSpec("C(Bc1(A))");
        assertLinkage("C", LINKAGE_ERROR, "B1");
        recompileSpec("C(Bc1(Ac0))", "A");
        assertLinkage("C", "A0", "B1");
    }
    @Test void test4() throws IOException, ReflectiveOperationException {
        compileSpec("C(B(Ac0))");
        assertLinkage("C", "A0", LINKAGE_ERROR);
        recompileSpec("C(Bc1(Ac0))", "B");
        assertLinkage("C", "B1", "B1");
    }
    @Test void test5() throws IOException, ReflectiveOperationException {
        compileSpec("C(B(A))");
        assertLinkage("C", LINKAGE_ERROR, LINKAGE_ERROR);
        recompileSpec("C(Bc1(Ac0))", "A", "B");
        assertLinkage("C", "B1", "B1");
    }
    @Test void test6() throws IOException, ReflectiveOperationException {
        compileSpec("C(Ac1(I))");
        assertLinkage("C", LINKAGE_ERROR, "A1");
        recompileSpec("C(Ac1(Id0))", "I");
        assertLinkage("C", "I0", "A1");
    }
    @Test void test7() throws IOException, ReflectiveOperationException {
        compileSpec("C(A(Id0))");
        assertLinkage("C", "I0", LINKAGE_ERROR);
        recompileSpec("C(Ac1(Id0))", "A");
        assertLinkage("C", "A1", "A1");
    }
    @Test void test8() throws IOException, ReflectiveOperationException {
        compileSpec("C(A(I))");
        assertLinkage("C", LINKAGE_ERROR, LINKAGE_ERROR);
        recompileSpec("C(Ac1(Id0))", "A", "I");
        assertLinkage("C", "A1", "A1");
    }
    @Test void test9() throws IOException, ReflectiveOperationException {
        compileSpec("C(Id1(J))");
        assertLinkage("C", LINKAGE_ERROR, "I1");
        recompileSpec("C(Id1(Jd0))", "J");
        assertLinkage("C", "J0", "I1");
    }
    @Test void test10() throws IOException, ReflectiveOperationException {
        compileSpec("C(I(Jd0))");
        assertLinkage("C", "J0", LINKAGE_ERROR);
        recompileSpec("C(Id1(Jd0))", "I");
        assertLinkage("C", "I1", "I1");
    }
    @Test void test11() throws IOException, ReflectiveOperationException {
        compileSpec("C(I(J))");
        assertLinkage("C", LINKAGE_ERROR, LINKAGE_ERROR);
        recompileSpec("C(Id1(Jd0))", "I", "J");
        assertLinkage("C", "I1", "I1");
    }
    @Test void test12() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(B(Ac0))");
        assertLinkage("C", "C2", LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(Bc1(Ac0))", "B");
        assertLinkage("C", "C2", "B1", "C2");
    }
    @Test void test13() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(B(Aa0))");
        assertLinkage("C", "C2", LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(Bc1(Aa0))", "B");
        assertLinkage("C", "C2", "B1", "C2");
    }
    @Test void test14() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(Bc1(A))");
        assertLinkage("C", LINKAGE_ERROR, "C2", "C2");
        recompileSpec("Cc2(Bc1(Ac0))", "A");
        assertLinkage("C", "A0", "C2", "C2");
    }
    @Test void test15() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(Ba1(A))");
        assertLinkage("C", LINKAGE_ERROR, "C2", "C2");
        recompileSpec("Cc2(Ba1(Ac0))", "A");
        assertLinkage("C", "A0", "C2", "C2");
    }
    @Test void test16() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(B(A))");
        assertLinkage("C", LINKAGE_ERROR, LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(Bc1(Ac0))", "B", "A");
        assertLinkage("C", "B1", "B1", "C2");
    }
    @Test void test17() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(A(Id0))");
        assertLinkage("C", "C2", LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(Ac1(Id0))", "A");
        assertLinkage("C", "C2", "A1", "C2");
    }
    @Test void test18() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(A(Ia0))");
        assertLinkage("C", "C2", LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(Ac1(Ia0))", "A");
        assertLinkage("C", "C2", "A1", "C2");
    }
    @Test void test19() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(Ac1(I))");
        assertLinkage("C", LINKAGE_ERROR, "C2", "C2");
        recompileSpec("Cc2(Ac1(Id0))", "I");
        assertLinkage("C", "I0", "C2", "C2");
    }
    @Test void test20() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(Aa1(I))");
        assertLinkage("C", LINKAGE_ERROR, "C2", "C2");
        recompileSpec("Cc2(Aa1(Id0))", "I");
        assertLinkage("C", "I0", "C2", "C2");
    }
    @Test void test21() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(A(I))");
        assertLinkage("C", LINKAGE_ERROR, LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(Ac1(Id0))", "A", "I");
        assertLinkage("C", "A1", "A1", "C2");
    }
    @Test void test22() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(J(Id0))");
        assertLinkage("C", "C2", LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(Jd1(Id0))", "J");
        assertLinkage("C", "C2", "J1", "C2");
    }
    @Test void test23() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(J(Ia0))");
        assertLinkage("C", "C2", LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(Jd1(Ia0))", "J");
        assertLinkage("C", "C2", "J1", "C2");
    }
    @Test void test24() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(Jd1(I))");
        assertLinkage("C", LINKAGE_ERROR, "C2", "C2");
        recompileSpec("Cc2(Jd1(Id0))", "I");
        assertLinkage("C", "I0", "C2", "C2");
    }
    @Test void test25() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(Ja1(I))");
        assertLinkage("C", LINKAGE_ERROR, "C2", "C2");
        recompileSpec("Cc2(Ja1(Id0))", "I");
        assertLinkage("C", "I0", "C2", "C2");
    }
    @Test void test26() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(J(I))");
        assertLinkage("C", LINKAGE_ERROR, LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(Jd1(Id0))", "J", "I");
        assertLinkage("C", "J1", "J1", "C2");
    }
    @Test void test27() throws IOException, ReflectiveOperationException {
        compileSpec("C(Ac1,I)");
        assertLinkage("C", LINKAGE_ERROR, "A1");
        recompileSpec("C(Ac1,Id0)", "I");
        assertLinkage("C", "I0", "A1");
    }
    @Test void test28() throws IOException, ReflectiveOperationException {
        compileSpec("C(A,Id0)");
        assertLinkage("C", "I0", LINKAGE_ERROR);
        recompileSpec("C(Ac1,Id0)", "A");
        assertLinkage("C", "I0", "A1");
    }
    @Test void test29() throws IOException, ReflectiveOperationException {
        compileSpec("C(A,I)");
        assertLinkage("C", LINKAGE_ERROR, LINKAGE_ERROR);
        recompileSpec("C(Ac1,Id0)", "A", "I");
        assertLinkage("C", "I0", "A1");
    }
    @Test void test30() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(Ac1,I)");
        assertLinkage("C", LINKAGE_ERROR, "C2", "C2");
        recompileSpec("Cc2(Ac1,Id0)", "I");
        assertLinkage("C", "I0", "C2", "C2");
    }
    @Test void test31() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(Aa1,I)");
        assertLinkage("C", LINKAGE_ERROR, "C2", "C2");
        recompileSpec("Cc2(Aa1,Id0)", "I");
        assertLinkage("C", "I0", "C2", "C2");
    }
    @Test void test32() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(A,Id0)");
        assertLinkage("C", "C2", LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(Ac1,Id0)", "A");
        assertLinkage("C", "C2", "A1", "C2");
    }
    @Test void test33() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(A,Ia0)");
        assertLinkage("C", "C2", LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(Ac1,Ia0)", "A");
        assertLinkage("C", "C2", "A1", "C2");
    }
    @Test void test34() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(A,I)");
        assertLinkage("C", LINKAGE_ERROR, LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(Ac1,Id0)", "A", "I");
        assertLinkage("C", "I0", "A1", "C2");
    }
    @Test void test35() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(Id0,J)");
        assertLinkage("C", "C2", LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(Id0,Jd1)", "J");
        assertLinkage("C", "C2", "J1", "C2");
    }
    @Test void test36() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(Ia0,J)");
        assertLinkage("C", "C2", LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(Ia0,Jd1)", "J");
        assertLinkage("C", "C2", "J1", "C2");
    }
    @Test void test37() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(I,J)");
        assertLinkage("C", LINKAGE_ERROR, LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(Id0,Jd1)", "I", "J");
        assertLinkage("C", "I0", "J1", "C2");
    }
    @Test void test38() throws IOException, ReflectiveOperationException {
        compileSpec("C(A(Id0),J(Id0))");
        assertLinkage("C", "I0", LINKAGE_ERROR);
        recompileSpec("C(Ac1(Id0),J(Id0))", "A");
        assertLinkage("C", "A1", "A1");
    }
    @Test void test39() throws IOException, ReflectiveOperationException {
        compileSpec("C(A(Id0),J(Id0))");
        assertLinkage("C", "I0", LINKAGE_ERROR);
        recompileSpec("C(A(Id0),Jd1(Id0))", "J");
        assertLinkage("C", "J1", "J1");
    }
    @Test void test40() throws IOException, ReflectiveOperationException {
        compileSpec("C(A(Id0),J(Id0))");
        assertLinkage("C", "I0", LINKAGE_ERROR);
        recompileSpec("C(Ac2(Id0),Jd1(Id0))", "A", "J");
        assertLinkage("C", "A2", "J1", "A2");
    }
    @Test void test41() throws IOException, ReflectiveOperationException {
        compileSpec("C(J(Id0),K(Id0))");
        assertLinkage("C", "I0", LINKAGE_ERROR);
        recompileSpec("C(Jd1(Id0),K(Id0))", "J");
        assertLinkage("C", "J1", "J1");
    }
    @Test void test42() throws IOException, ReflectiveOperationException {
        compileSpec("C(Ac2(Id0),J(Id0))");
        assertLinkage("C", "A2", LINKAGE_ERROR, "A2");
        recompileSpec("C(Ac2(Id0),Jd1(Id0))", "J");
        assertLinkage("C", "A2", "J1", "A2");
    }
    @Test void test43() throws IOException, ReflectiveOperationException {
        compileSpec("C(Ac2(Ia0),J(Ia0))");
        assertLinkage("C", "A2", LINKAGE_ERROR, "A2");
        recompileSpec("C(Ac2(Ia0),Jd1(Ia0))", "J");
        assertLinkage("C", "A2", "J1", "A2");
    }
    @Test void test44() throws IOException, ReflectiveOperationException {
        compileSpec("C(A(Id0),Jd1(Id0))");
        assertLinkage("C", "J1", "J1", LINKAGE_ERROR);
        recompileSpec("C(Ac2(Id0),Jd1(Id0))", "A");
        assertLinkage("C", "A2", "J1", "A2");
    }
    @Test void test45() throws IOException, ReflectiveOperationException {
        compileSpec("C(A(Ia0),Jd1(Ia0))");
        assertLinkage("C", "J1", "J1", LINKAGE_ERROR);
        recompileSpec("C(Ac2(Ia0),Jd1(Ia0))", "A");
        assertLinkage("C", "A2", "J1", "A2");
    }
    @Test void test46() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(A(Id0),J(Id0))");
        assertLinkage("C", "C2", LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(Ac1(Id0),J(Id0))", "A");
        assertLinkage("C", "C2", "A1", "C2");
    }
    @Test void test47() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(A(Ia0),J(Ia0))");
        assertLinkage("C", "C2", LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(Ac1(Ia0),J(Ia0))", "A");
        assertLinkage("C", "C2", "A1", "C2");
    }
    @Test void test48() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(A(Id0),J(Id0))");
        assertLinkage("C", "C2", LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(A(Id0),Jd1(Id0))", "J");
        assertLinkage("C", "C2", "J1", "C2");
    }
    @Test void test49() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(A(Ia0),J(Ia0))");
        assertLinkage("C", "C2", LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(A(Ia0),Jd1(Ia0))", "J");
        assertLinkage("C", "C2", "J1", "C2");
    }
    @Test void test50() throws IOException, ReflectiveOperationException {
        compileSpec("Cc3(A(Id0),J(Id0))");
        assertLinkage("C", "C3", LINKAGE_ERROR, LINKAGE_ERROR, "C3");
        recompileSpec("Cc3(Ac1(Id0),Jd2(Id0))", "A", "J");
        assertLinkage("C", "C3", "A1", "J2", "C3");
    }
    @Test void test51() throws IOException, ReflectiveOperationException {
        compileSpec("Cc3(A(Ia0),J(Ia0))");
        assertLinkage("C", "C3", LINKAGE_ERROR, LINKAGE_ERROR, "C3");
        recompileSpec("Cc3(Ac1(Ia0),Jd2(Ia0))", "A", "J");
        assertLinkage("C", "C3", "A1", "J2", "C3");
    }
    @Test void test52() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(J(Id0),K(Id0))");
        assertLinkage("C", "C2", LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(Jd1(Id0),K(Id0))", "J");
        assertLinkage("C", "C2", "J1", "C2");
    }
    @Test void test53() throws IOException, ReflectiveOperationException {
        compileSpec("Cc2(J(Ia0),K(Ia0))");
        assertLinkage("C", "C2", LINKAGE_ERROR, "C2");
        recompileSpec("Cc2(Jd1(Ia0),K(Ia0))", "J");
        assertLinkage("C", "C2", "J1", "C2");
    }
    @Test void test54() throws IOException, ReflectiveOperationException {
        compileSpec("Cc3(J(Id0),K(Id0))");
        assertLinkage("C", "C3", LINKAGE_ERROR, LINKAGE_ERROR, "C3");
        recompileSpec("Cc3(Jd1(Id0),Kd2(Id0))", "J", "K");
        assertLinkage("C", "C3", "J1", "K2", "C3");
    }
    @Test void test55() throws IOException, ReflectiveOperationException {
        compileSpec("Cc3(J(Ia0),K(Ia0))");
        assertLinkage("C", "C3", LINKAGE_ERROR, LINKAGE_ERROR, "C3");
        recompileSpec("Cc3(Jd1(Ia0),Kd2(Ia0))", "J", "K");
        assertLinkage("C", "C3", "J1", "K2", "C3");
    }
    @Test void test56() throws IOException, ReflectiveOperationException {
        compileSpec("Cc3(Ac1(Id0),J(Id0))");
        assertLinkage("C", "C3", "C3", LINKAGE_ERROR, "C3");
        recompileSpec("Cc3(Ac1(Id0),Jd2(Id0))", "J");
        assertLinkage("C", "C3", "C3", "J2", "C3");
    }
    @Test void test57() throws IOException, ReflectiveOperationException {
        compileSpec("Cc3(Ac1(Ia0),J(Ia0))");
        assertLinkage("C", "C3", "C3", LINKAGE_ERROR, "C3");
        recompileSpec("Cc3(Ac1(Ia0),Jd2(Ia0))", "J");
        assertLinkage("C", "C3", "C3", "J2", "C3");
    }
    @Test void test58() throws IOException, ReflectiveOperationException {
        compileSpec("Cc3(Aa1(Id0),J(Id0))");
        assertLinkage("C", "C3", "C3", LINKAGE_ERROR, "C3");
        recompileSpec("Cc3(Aa1(Id0),Jd2(Id0))", "J");
        assertLinkage("C", "C3", "C3", "J2", "C3");
    }
    @Test void test59() throws IOException, ReflectiveOperationException {
        compileSpec("Cc3(Aa1(Ia0),J(Ia0))");
        assertLinkage("C", "C3", "C3", LINKAGE_ERROR, "C3");
        recompileSpec("Cc3(Aa1(Ia0),Jd2(Ia0))", "J");
        assertLinkage("C", "C3", "C3", "J2", "C3");
    }
    @Test void test60() throws IOException, ReflectiveOperationException {
        compileSpec("Cc3(A(Id0),Jd2(Id0))");
        assertLinkage("C", "C3", LINKAGE_ERROR, "C3", "C3");
        recompileSpec("Cc3(Ac1(Id0),Jd2(Id0))", "A");
        assertLinkage("C", "C3", "A1", "C3", "C3");
    }
    @Test void test61() throws IOException, ReflectiveOperationException {
        compileSpec("Cc3(A(Ia0),Jd2(Ia0))");
        assertLinkage("C", "C3", LINKAGE_ERROR, "C3", "C3");
        recompileSpec("Cc3(Ac1(Ia0),Jd2(Ia0))", "A");
        assertLinkage("C", "C3", "A1", "C3", "C3");
    }
    @Test void test62() throws IOException, ReflectiveOperationException {
        compileSpec("Cc3(A(Id0),Ja2(Id0))");
        assertLinkage("C", "C3", LINKAGE_ERROR, "C3", "C3");
        recompileSpec("Cc3(Ac1(Id0),Ja2(Id0))", "A");
        assertLinkage("C", "C3", "A1", "C3", "C3");
    }
    @Test void test63() throws IOException, ReflectiveOperationException {
        compileSpec("Cc3(A(Ia0),Ja2(Ia0))");
        assertLinkage("C", "C3", LINKAGE_ERROR, "C3", "C3");
        recompileSpec("Cc3(Ac1(Ia0),Ja2(Ia0))", "A");
        assertLinkage("C", "C3", "A1", "C3", "C3");
    }
    @Test void test64() throws IOException, ReflectiveOperationException {
        compileSpec("Cc3(Jd1(Id0),K(Id0))");
        assertLinkage("C", "C3", "C3", LINKAGE_ERROR, "C3");
        recompileSpec("Cc3(Jd1(Id0),Kd2(Id0))", "K");
        assertLinkage("C", "C3", "C3", "K2", "C3");
    }
    @Test void test65() throws IOException, ReflectiveOperationException {
        compileSpec("Cc3(Jd1(Ia0),K(Ia0))");
        assertLinkage("C", "C3", "C3", LINKAGE_ERROR, "C3");
        recompileSpec("Cc3(Jd1(Ia0),Kd2(Ia0))", "K");
        assertLinkage("C", "C3", "C3", "K2", "C3");
    }
    @Test void test66() throws IOException, ReflectiveOperationException {
        compileSpec("Cc3(Jd1(Id0),K(Id0))");
        assertLinkage("C", "C3", "C3", LINKAGE_ERROR, "C3");
        recompileSpec("Cc3(Jd1(Id0),Kd2(Id0))", "K");
        assertLinkage("C", "C3", "C3", "K2", "C3");
    }
    @Test void test67() throws IOException, ReflectiveOperationException {
        compileSpec("Cc3(Jd1(Ia0),K(Ia0))");
        assertLinkage("C", "C3", "C3", LINKAGE_ERROR, "C3");
        recompileSpec("Cc3(Jd1(Ia0),Kd2(Ia0))", "K");
        assertLinkage("C", "C3", "C3", "K2", "C3");
    }
    @Test void testA1() throws IOException, ReflectiveOperationException {
        compileSpec("C(Id0)");
        assertLinkage("C", "I0");
    }
    @Test void testA2() throws IOException, ReflectiveOperationException {
        compileSpec("C(A(Id0))");
        assertLinkage("C", "I0");
    }
    @Test void testA3() throws IOException, ReflectiveOperationException {
        compileSpec("C(A(Id0),J)");
        assertLinkage("C", "I0");
    }
    @Test void testA4() throws IOException, ReflectiveOperationException {
        compileSpec("D(C(Id0),Jd0(Id0))");
        assertLinkage("D", "J0");
        assertLinkage("C", "I0");
    }
    @Test void testA5() throws IOException, ReflectiveOperationException {
        compileSpec("C(A(Id0),Jd0)", "compiler.err.types.incompatible");
    }
    @Test void testA6() throws IOException, ReflectiveOperationException {
        compileSpec("C(A(Ia0,Jd0))", "compiler.err.does.not.override.abstract", "compiler.err.types.incompatible");
    }
    @Test void testA7() throws IOException, ReflectiveOperationException {
        compileSpec("C(A(Id0,Jd0))", "compiler.err.types.incompatible");
    }
    @Test void testA8() throws IOException, ReflectiveOperationException {
        compileSpec("C(A(Ia0),J)", "compiler.err.does.not.override.abstract");
    }
    @Test void testA9() throws IOException, ReflectiveOperationException {
        compileSpec("C(Ac0(Id0))");
        assertLinkage("C", "A0");
    }
    @Test void testA10() throws IOException, ReflectiveOperationException {
        compileSpec("C(Aa0,I)", "compiler.err.does.not.override.abstract");
    }
    @Test void testA11() throws IOException, ReflectiveOperationException {
        compileSpec("C(Ac0,Id0)");
        assertLinkage("C", "A0");
    }
    @Test void testB2() throws IOException, ReflectiveOperationException {
        compileSpec("Cc1(Ac0)");
        assertLinkage("C", "C1", "C1");
    }
    @Test void testB5() throws IOException, ReflectiveOperationException {
        compileSpec("Cc1(Aa1(Id0))");
        assertLinkage("C", "C1", "C1");
    }
    @Test void testB6() throws IOException, ReflectiveOperationException {
        compileSpec("C(Ac1(Id0))");
        assertLinkage("C", "A1", "A1");
    }
    @Test void testB7() throws IOException, ReflectiveOperationException {
        compileSpec("Cc1(Id0)");
        assertLinkage("C", "C1", "C1");
    }
    @Test void testB8() throws IOException, ReflectiveOperationException {
        compileSpec("C(Jd1(Id0))");
        assertLinkage("C", "J1", "J1");
    }
}
