import java.lang.annotation.*;

class IndexArray {
    int[] var;
    int a = var;
    [1];
}

@Target(ElementType.TYPE_USE) @interface A {
}
