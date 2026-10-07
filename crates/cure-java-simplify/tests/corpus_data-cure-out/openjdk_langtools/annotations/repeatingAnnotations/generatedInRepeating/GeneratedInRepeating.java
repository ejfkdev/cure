import java.lang.annotation.Repeatable;

@Annot(Gen.class)
@Annot(Gen.class)
public class GeneratedInRepeating {
}

@Repeatable(AnnotContainer.class) @interface Annot {
    public Class<?> value();
}

@interface AnnotContainer {
    public Annot[] value();
}
