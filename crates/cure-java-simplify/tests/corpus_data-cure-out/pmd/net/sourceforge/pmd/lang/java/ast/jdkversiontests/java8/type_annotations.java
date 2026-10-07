@Target(ElementType.TYPE_USE) @interface NonNull {
}

class AnnotatedArrayType {
    @NonNull int[][] field1;
    @NonNull int[][] field2;
    private @NonNull int array2[][];
    public String m2() 
        @NonNull[]@NonNull[] { return null; }
            public String@NonNull[]@NonNull[] m2a() { return null; }
            public void run() {
                for (String a@NonNull[] : m2()) {
                }
            }
            void vararg(@NonNull String @NonNull [] @NonNull ... vararg2) { }
            public void vararg2(@NonNull int @NonNull ... vararg) {}
            public void vararg3(@NonNull int[] @NonNull ... vararg) {}
}

@Target({
    ElementType.FIELD, ElementType.LOCAL_VARIABLE, ElementType.PARAMETER,
    ElementType.TYPE_PARAMETER, ElementType.TYPE_USE}) @interface TypeAnnotation {
}

class Rectangle2D {
    class Double {
    }
}

class TypeAnnotations {
    private Map.Entry entry;
    boolean isNonNull = "string" instanceof String;
    public final Rectangle2D.Double getRect1() {
        return new Rectangle2D.Double();
    }
    public final Rectangle2D.Double getRect2() {
        return new Rectangle2D.Double();
    }
    public final Rectangle2D.Double getRect3() {
        Rectangle2D.Double rect = null;
        int[][] i = new int @TypeAnnotation [1] @TypeAnnotation[];
        i = new @TypeAnnotation
        int [1] @TypeAnnotation[];
        return rect;
    }
    class Outer {
        class Inner {
            class Inner2 {
            }
        }
        class GInner<X> {
            class GInner2<Y, Z> {
            }
        }
        class Static {
        }
        class GStatic<X, Y> {
            class GStatic2<Z> {
            }
        }
    }
    class MyList<K> {
    }
    class Test1 {
        @TypeAnnotation Outer.GInner<@TypeAnnotation MyList<@TypeAnnotation Object @TypeAnnotation[] @TypeAnnotation[]>>.GInner2<@TypeAnnotation Integer, @TypeAnnotation Object>[][] f4arrtop;
    }
}
