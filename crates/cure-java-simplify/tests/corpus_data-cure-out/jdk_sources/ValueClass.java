package java.lang;

import jdk.internal.value.Deserializer;
import jdk.internal.vm.annotation.IntrinsicCandidate;
import java.lang.constant.Constable;
import java.lang.constant.ConstantDescs;
import java.lang.constant.DynamicConstantDesc;
import java.util.Optional;

@jdk.internal.ValueBased
// See doc/value-class-preview.md for an overview of value class generation
public final class Boolean implements java.io.Serializable, Comparable<Boolean>, Constable {
    public static final Boolean TRUE = new Boolean(true);
    public static final Boolean FALSE = new Boolean(false);
    public static final Class<Boolean> TYPE = Class.getPrimitiveClass("boolean");
    private final boolean value;
    @java.io.Serial
    private static final long serialVersionUID = -3665804199014368530L;
    @Deprecated(since="9")
    @Deserializer("value")
    public Boolean(boolean value) {
        this.value = value;
    }
    @Deprecated(since="9")
    public Boolean(String s) {
        this(parseBoolean(s));
    }
    public static boolean parseBoolean(String s) {
        return "true".equalsIgnoreCase(s);
    }
    @IntrinsicCandidate
    public boolean booleanValue() {
        return value;
    }
    @IntrinsicCandidate
    public static Boolean valueOf(boolean b) {
        return b ? TRUE : FALSE;
    }
    public static Boolean valueOf(String s) {
        return parseBoolean(s) ? TRUE : FALSE;
    }
    public static String toString(boolean b) {
        return String.valueOf(b);
    }
    @Override
    public String toString() {
        return String.valueOf(value);
    }
    @Override
    public int hashCode() {
        return Boolean.hashCode(value);
    }
    public static int hashCode(boolean value) {
        return value ? 1231 : 1237;
    }
    public boolean equals(Object obj) {
        return obj instanceof Boolean b && value == b.booleanValue();
    }
    public static boolean getBoolean(String name) {
        return name != null && !name.isEmpty() && parseBoolean(System.getProperty(name));
    }
    public int compareTo(Boolean b) {
        return compare(this.value, b.value);
    }
    public static int compare(boolean x, boolean y) {
        return x == y ? 0 : x ? 1 : -1;
    }
    public static boolean logicalAnd(boolean a, boolean b) {
        return a && b;
    }
    public static boolean logicalOr(boolean a, boolean b) {
        return a || b;
    }
    public static boolean logicalXor(boolean a, boolean b) {
        return a ^ b;
    }
    @Override
    public Optional<DynamicConstantDesc<Boolean>> describeConstable() {
        return Optional.of(ConstantDescs.TRUE);
    }
}
