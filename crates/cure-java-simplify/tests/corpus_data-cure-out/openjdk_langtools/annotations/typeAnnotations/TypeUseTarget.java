import java.lang.annotation.Target;
import java.lang.annotation.ElementType;

@A class TypeUseTarget<K extends @A Object> {
    @A String[] field;
    @A String test(@A TypeUseTarget<K> this, @A String param, @A String... vararg) {
        @A Object o = new @A String @A [3];
        TypeUseTarget<@A String> target;
        return (String) null;
    }
    <K> String genericMethod(K k) {
        return null;
    }
    @Decl <K> String genericMethod1(K k) {
        return null;
    }
    @A @Decl <K> String genericMethod2(K k) {
        return null;
    }
    @Decl @A <K> String genericMethod3(K k) {
        return null;
    }
    <K> String genericMethod4(K k) {
        return null;
    }
    <K> String genericMethod5(K k) {
        return null;
    }
}

@A interface MyInterface {
}

@A @interface MyAnnotation {
}

@Target(ElementType.TYPE_USE) @interface A {
}

@interface Decl {
}
