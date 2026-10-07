package spoon.test.position.testclasses;

import java.lang.Deprecated;
import java.lang.Class;

public class FooSourceFragments {
    void m1(int x) {
        if (x > 0) {
            this.getClass();
        }
    }
    void m2(int x) {
        if (x > 0) {
            this.getClass();
        }
    }
    public
	@Deprecated //c1 ends with tab and space	 
	static <T, U> T m3(U param, @Deprecated int p2) {
        return null;
    }
    void m4() {
        label:
            while (true) ;
    }
    void m5(double f) {
        f = 7.2;
    }
}
