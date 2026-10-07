import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Retention;

class NoTargetOnTypeParameterDeclaration {
    @Retention(RetentionPolicy.RUNTIME) @interface A {
    }
    class B<@A X> {
        <@A Y> void f() {}
    }
}
