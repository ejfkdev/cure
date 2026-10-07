package spoon.test.query_function.testclasses;

import java.io.IOException;
import java.io.Reader;
import java.io.StringReader;
import java.util.function.Consumer;
import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.assertTrue;

public class VariableReferencesModelTest {
    int field = 15;
    @Test
	public void localVarsInNestedBlocks() {
        assertTrue(this.field == 15);
        {
            assertTrue(true);
            int f1;
            int f2;
            int f3;
            int f4;
            assertTrue(true);
        }
        int field = 3;
        assertTrue(field == 3);
        assertTrue(field == 3);
        assertTrue(this.field == 15);
    }
    @Test
	public void localVarsInTryCatch() {
        try {
            assertTrue(true);
            throw new IllegalArgumentException();
        } catch (IllegalArgumentException e) {
            assertTrue(field == 15);
            assertTrue(true);
            assertTrue(true);
        } catch (Exception field) {
            field.getMessage();
        }
    }
    @Test
	public void localVarsInWhile() {
        while (true) {
            assertTrue(true);
            break;
        }
        assertTrue(true);
    }
    @Test
	public void localVarsInFor() {
        for (int field = 10; field == 10; ) {
            assertTrue(field == 10);
            break;
        }
        assertTrue(true);
    }
    @Test
	public void localVarsInSwitch() {
        switch (7) {
            case 7:
                int field = 12;
                assertTrue(field == 12);
                break;
        }
        assertTrue(true);
    }
    @Test
	public void localVarsInTryWithResource() throws IOException {
        try (Reader field = new StringReader("")) {
            field.toString();
        }
    }
    @Test
	public void checkParameter() {
        parameter(16);
    }
    private void parameter(int field) {
        assertTrue(field == 16);
        assertTrue(field == 16);
        while (true) {
            assertTrue(field == 16);
            break;
        }
    }
    @Test
	public void parameterInLambdaWithBody() {
        Consumer<Integer> fnc = (field) -> {
            assertTrue(field == 17);
        };
        fnc.accept(17);
    }
    @Test
	public void parameterInLambdaWithExpression() {
        Consumer<Integer> fnc = (field) -> assertTrue(field == 18);
        fnc.accept(18);
    }
    @Test
	public void localVarInLambda() {
        Runnable fnc = () -> {
            assertTrue(true);
        };
        fnc.run();
        Runnable fnc2 = () -> {
            assertTrue(true);
        };
        fnc2.run();
    }
    static {
        assertTrue(true);
    }
    {
        assertTrue(true);
    }
    @Test
	public void localVarInNestedClass() {
        int field = 23;
        assertTrue(field == 23);
        new Runnable() {
			@Override
			public void run() {
				{
					int field = 24;
					assertTrue(field == 24);
				}
				assertTrue(field == 23);
				int field = 25;
				assertTrue(field == 25);
			}
		}.run();
        assertTrue(field == 23);
    }
    @Test
	public void localVarInNestedClass2() {
        int field = 26;
        assertTrue(field == 26);
        new Runnable() {
			int field = 27;
			@Override
			public void run() {
				{
					int field = 36;
					assertTrue(field == 36);
				}
				assertTrue(field == 27);
				int field = 28;
				assertTrue(field == 28);
				assertTrue(this.field == 27);
			}
		}.run();
        assertTrue(field == 26);
    }
    class A {
        int field = 29;
    }
    abstract class B extends A {
        abstract void run();
    }
    @Test
	public void localVarInNestedClass4() {
        int field = 30;
        assertTrue(field == 30);
        new B() {
			@Override
			public void run() {
				{
					int field = 31;
					assertTrue(field == 31);
				}
				assertTrue(field == 29);
				int field = 32;
				assertTrue(field == 32);
				assertTrue(this.field == 29);
			}
		}.run();
        assertTrue(field == 30);
    }
    @Test
	public void localVarInNestedClass5() {
        int field = 33;
        assertTrue(field == 33);
        class Local {
        			{
        				{
        					int field = 34;
        					assertTrue(field == 34);
        				}
        				assertTrue(field == 33);
        				int field = 35;
        				assertTrue(field == 35);
        			}
        		}
        		new Local();
        assertTrue(field == 33);
    }
    @Test
	public void localVarInNestedClass6() {
        int field = 37;
        assertTrue(field == 37);
        class Local {
        			int field = 38;
        			void method(int field) {
        				assertTrue(field == 39);
        				assertTrue(this.field == 38);
        			}
        		}
        		new Local().method(39);
        assertTrue(field == 37);
    }
    private static final int maxValue = 39;
}
