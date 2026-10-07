import java.math.BigInteger;
import java.security.cert.Certificate;
import java.security.interfaces.DSAPublicKey;
import java.security.interfaces.RSAKey;

public class Jep513_FlexibleConstructorBodies {
    public class PositiveBigInteger extends BigInteger {
        public PositiveBigInteger(long value) {
            if (value <= 0) 
                throw new IllegalArgumentException("value must be positive");
            super(String.valueOf(value));
        }
    }
    public class Super {
        public Super(byte[] bytes) {}
    }
    public class Sub extends Super {
        public Sub(Certificate certificate) {
            var publicKey = certificate.getPublicKey();
            if (publicKey == null) 
                throw new NullPointerException();
            super(switch (publicKey) {
                case RSAKey rsaKey -> rsaKey.getModulus().toByteArray();
                case DSAPublicKey dsaKey -> dsaKey.getY().toByteArray();
                default -> new byte[0];
            });
        }
    }
    public class C {
        private final int i;
        public C(int i) {
            this.i = i;
        }
    }
    public class Super2 {
        private final C x;
        private final C y;
        public Super2(C x, C y) {
            this.x = x;
            this.y = y;
        }
    }
    public class Sub2 extends Super2 {
        public Sub2(int i) {
            var x = new C(i);
            super(x, x);
        }
    }
    class Outer {
        int i;
        void hello() {
            System.out.println("Hello");
        }
        class Inner {
            int j;
            Inner() {
                var y = Outer.this.i;
                hello();
                Outer.this.hello();
                super();
            }
        }
    }
    class Super3 {
        Super3() {
            overriddenMethod();
        }
        void overriddenMethod() {
            System.out.println("hello");
        }
    }
    class Sub3 extends Super3 {
        final int x;
        Sub3(int x) {
            this.x = x;
            super();
        }
        @Override void overriddenMethod() {
            System.out.println(x);
        }
    }
}
