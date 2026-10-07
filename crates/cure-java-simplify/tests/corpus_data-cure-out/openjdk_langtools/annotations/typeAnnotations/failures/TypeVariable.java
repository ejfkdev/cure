import java.lang.annotation.*;

class TypeVariable {
    <TV extends  @TA Object> TV cast(TV p) {
        return p;
    }
}

@Target({ElementType.TYPE_USE, ElementType.TYPE_PARAMETER}) @interface TA {
}
