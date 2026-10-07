import java.lang.annotation.*;

class DefaultScope {
    Parameterized<String, String> unannotated;
    Parameterized<@A String, String> firstTypeArg;
    Parameterized<String, @A String> secondTypeArg;
    Parameterized<@A String, @B String> bothTypeArgs;
    Parameterized<@A Parameterized<@A String, @B String>, @B String> nestedParameterized;
    @A String[] array1;
    @A String[] array1Deep;
    @A String[][] array2;
    @A String[][] array2Deep;
    String[][] array2First;
    String[][] array2Second;
    String array2FirstOld[];
    String array2SecondOld[][];
}

class ModifiedScoped {
    public final Parameterized<String, String> unannotated = null;
    public final Parameterized<@A String, String> firstTypeArg = null;
    public final Parameterized<String, @A String> secondTypeArg = null;
    public final Parameterized<@A String, @B String> bothTypeArgs = null;
    public final Parameterized<@A Parameterized<@A String, @B String>, @B String> nestedParameterized = null;
    public final @A String[] array1 = null;
    public final @A String[] array1Deep = null;
    public final @A String[][] array2 = null;
    public final @A String[][] array2Deep = null;
    public final String[][] array2First = null;
    public final String[][] array2Second = null;
}

class Parameterized<K, V> {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface A {
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface B {
}
