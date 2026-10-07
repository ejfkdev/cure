import java.lang.annotation.*;

class DefaultScope {
    void parameterized() {
        Parameterized<String, String> unannotated;
        Parameterized<@A String, String> firstTypeArg;
        Parameterized<String, @A String> secondTypeArg;
        Parameterized<@A String, @B String> bothTypeArgs;
        Parameterized<@A Parameterized<@A String, @B String>, @B String> nestedParameterized;
    }
    void arrays() {
        @A String [] array1;
        @A String @B [] array1Deep;
        @A String [] [] array2;
        @A String @A [] @B [] array2Deep;
        String[][] array2First;
        String[][] array2Second;
    }
}

class ModifiedVars {
    void parameterized() {
        Parameterized<@A Parameterized<@A String, @B String>, @B String> nestedParameterized = null;
    }
    void arrays() {
        String[][] array2Second = null;
    }
}

class Parameterized<K, V> {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface A {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface B {
}
