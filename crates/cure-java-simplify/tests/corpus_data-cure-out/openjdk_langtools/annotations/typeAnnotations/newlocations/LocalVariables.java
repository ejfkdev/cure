import java.lang.annotation.*;

class DefaultScope {
    void parameterized() {}
    void arrays() {
        @A String [] array1;
        @A String @B [] array1Deep;
        @A String [] [] array2;
        @A String @A [] @B [] array2Deep;
    }
}

class ModifiedVars {
    void parameterized() {}
    void arrays() {}
}

class Parameterized<K, V> {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface A {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface B {
}
