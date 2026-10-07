package com.github.javaparser.ast.type;

import com.github.javaparser.Range;
import com.github.javaparser.ast.nodeTypes.NodeWithAnnotations;
import com.github.javaparser.ast.visitor.GenericVisitor;
import com.github.javaparser.ast.visitor.VoidVisitor;
import static com.github.javaparser.utils.Utils.ensureNotNull;
import java.util.HashMap;

public final class PrimitiveType extends Type<PrimitiveType> implements NodeWithAnnotations<PrimitiveType> {
    public static final PrimitiveType BYTE_TYPE = new PrimitiveType(Primitive.Byte);
    public static final PrimitiveType SHORT_TYPE = new PrimitiveType(Primitive.Short);
    public static final PrimitiveType INT_TYPE = new PrimitiveType(Primitive.Int);
    public static final PrimitiveType LONG_TYPE = new PrimitiveType(Primitive.Long);
    public static final PrimitiveType FLOAT_TYPE = new PrimitiveType(Primitive.Float);
    public static final PrimitiveType DOUBLE_TYPE = new PrimitiveType(Primitive.Double);
    public static final PrimitiveType BOOLEAN_TYPE = new PrimitiveType(Primitive.Boolean);
    public static final PrimitiveType CHAR_TYPE = new PrimitiveType(Primitive.Char);
    public enum Primitive {
        Boolean ("Boolean"), Char    ("Character"), Byte    ("Byte"), Short   ("Short"), Int     ("Integer"), Long    ("Long"), Float   ("Float"), Double  ("Double");
        final String nameOfBoxedType;
        public ClassOrInterfaceType toBoxedType() {
            return new ClassOrInterfaceType(nameOfBoxedType);
        }
        Primitive(String nameOfBoxedType) {
            this.nameOfBoxedType = nameOfBoxedType;
        }
    }
    static final HashMap<String, Primitive> unboxMap = new HashMap<>();
    static {
        for (Primitive unboxedType : Primitive.values()) {
            unboxMap.put(unboxedType.nameOfBoxedType, unboxedType);
        }
    }
    private Primitive type;
    public PrimitiveType() {}
    public PrimitiveType(final Primitive type) {
        this.type = type;
    }
    public PrimitiveType(Range range, final Primitive type) {
        super(range);
        setType(type);
    }
    @Override public <R, A> R accept(final GenericVisitor<R, A> v, final A arg) {
        return v.visit(this, arg);
    }
    @Override public <A> void accept(final VoidVisitor<A> v, final A arg) {
        v.visit(this, arg);
    }
    public Primitive getType() {
        return type;
    }
    public ClassOrInterfaceType toBoxedType() {
        return type.toBoxedType();
    }
    public PrimitiveType setType(final Primitive type) {
        this.type = type;
        return this;
    }
}
