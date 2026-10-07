import java.io.File;
import java.util.function.Supplier;
import org.checkerframework.checker.nullness.qual.NonNull;

public class FullTypeAnnotations {
    private String myString;
    Map<@NonNull String, @NonEmpty List<@Readonly Document>> files;
    {
        o.m("...");
    }
    class Folder<F extends @Existing File> {
    }
    Collection<? super @Existing File> field;
    class UnmodifiableList<T> implements @Readonly List<@Readonly T> {
    }
    void monitorTemperature() throws @Critical TemperatureException {}
    {
        new @Interned MyObject();
        new @NonEmpty @Readonly List<String>(myNonEmptyStringSet);
        myVar.new @Tainted NestedClass();

                // For generic constructors(JLS §8.8.4),the annotation follows the explicit type arguments (JLS §15.9):
        new <String>@Interned MyObject();
    }
    Map.Entry mapField;
    {
        myString = (String) myObject;
        x = (Type1 & Type2) y;
    }
    boolean isNonNull = myString instanceof String;
    {
        Supplier<@Vernal Date> sup = Arrays::sort;
    }
    @Readonly Document[][] docs1;
    [2][12]; // array of arrays of read-only documents
    Document[][] docs2;
    [2][12]; // read-only array of arrays of documents
    Document[][] docs3 = new Document[2];
    [12]; // array of read-only arrays of documents
    Document[] docs4[];
    [2][12]; // read-only array of arrays of documents
    Document[] docs5[] = new Document[2];
    [12]; // array of read-only arrays of documents
    {
        @Readonly Document[][] docs1 = new @Readonly Document[2][12]; // array of arrays of read-only documents
        Document @Readonly [][] docs2 = new Document@Readonly[2][12]; // read-only array of arrays of documents
        Document[][] docs3 = new Document[2];
        @Readonly[12]; // array of read-only arrays of documents
        Document[] docs4;
        @Readonly[] = new Document@Readonly[2][12]; // read-only array of arrays of documents
        Document[][] docs5 = new Document[2];
        @Readonly[12]; // array of read-only arrays of documents
    }
    class MyClass {
        public String toString(@Readonly MyClass this) {}
        public boolean equals(Object... other) 
            @K[][]{ }
                    MyClass(Object @Readonly [] @ß... other) { }
    }
}
