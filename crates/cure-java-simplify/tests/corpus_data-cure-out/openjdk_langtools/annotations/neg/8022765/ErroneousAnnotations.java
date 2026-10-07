class ErroneousAnnotations {
    @Undefined //no "is not an annotation type error"
    private int f1;
    @String //produce "is not an annotation type error"
    private int f2;
    @Annot(@Undefined)
    private int f3;
    @Annot(@String)
    private int f4;
    @Primitive(@Undefined)
    private int f5;
    @Primitive(@String)
    private int f6;
    @PrimitiveWrap(@PrimitiveImpl)
    private int f7;
    @interface Annot {
        Undefined value();
    }
    @interface PrimitiveWrap {
        Primitive value();
    }
    @interface Primitive {
        int value();
    }
    interface PrimitiveImpl extends Primitive {
    }
}
