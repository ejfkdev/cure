package jdk.internal.util;

import java.util.Arrays;
import java.util.Collection;
import java.util.Objects;
import jdk.internal.access.JavaLangAccess;
import jdk.internal.access.SharedSecrets;
import jdk.internal.misc.Unsafe;
import jdk.internal.vm.annotation.IntrinsicCandidate;

public class ArraysSupport {
    static final Unsafe U = Unsafe.getUnsafe();
    private static final boolean BIG_ENDIAN = U.isBigEndian();
    public static final int LOG2_ARRAY_BOOLEAN_INDEX_SCALE = exactLog2(Unsafe.ARRAY_BOOLEAN_INDEX_SCALE);
    public static final int LOG2_ARRAY_BYTE_INDEX_SCALE = exactLog2(Unsafe.ARRAY_BYTE_INDEX_SCALE);
    public static final int LOG2_ARRAY_CHAR_INDEX_SCALE = exactLog2(Unsafe.ARRAY_CHAR_INDEX_SCALE);
    public static final int LOG2_ARRAY_SHORT_INDEX_SCALE = exactLog2(Unsafe.ARRAY_SHORT_INDEX_SCALE);
    public static final int LOG2_ARRAY_INT_INDEX_SCALE = exactLog2(Unsafe.ARRAY_INT_INDEX_SCALE);
    public static final int LOG2_ARRAY_LONG_INDEX_SCALE = exactLog2(Unsafe.ARRAY_LONG_INDEX_SCALE);
    public static final int LOG2_ARRAY_FLOAT_INDEX_SCALE = exactLog2(Unsafe.ARRAY_FLOAT_INDEX_SCALE);
    public static final int LOG2_ARRAY_DOUBLE_INDEX_SCALE = exactLog2(Unsafe.ARRAY_DOUBLE_INDEX_SCALE);
    private static final int LOG2_BYTE_BIT_SIZE = exactLog2(Byte.SIZE);
    private static int exactLog2(int scale) {
        if ((scale & scale - 1) != 0) 
            throw new Error("data type scale not a power of two");
        return Integer.numberOfTrailingZeros(scale);
    }
    private ArraysSupport() {}
    @IntrinsicCandidate
    public static int vectorizedMismatch(Object a, long aOffset, Object b, long bOffset, int length, int log2ArrayIndexScale) {
        int log2ValuesPerWidth = LOG2_ARRAY_LONG_INDEX_SCALE - log2ArrayIndexScale;
        int wi = 0;
        for (; wi < length >> log2ValuesPerWidth; wi++) {
            long bi = (long) wi << LOG2_ARRAY_LONG_INDEX_SCALE;
            long av = U.getLongUnaligned(a, aOffset + bi);
            long bv = U.getLongUnaligned(b, bOffset + bi);
            if (av != bv) {
                return (wi << log2ValuesPerWidth) + (Long.numberOfLeadingZeros(av ^ bv) >> LOG2_BYTE_BIT_SIZE + log2ArrayIndexScale);
            }
        }
        int tail = length - (wi << log2ValuesPerWidth);
        if (log2ArrayIndexScale < LOG2_ARRAY_INT_INDEX_SCALE) {
            int wordTail = 1 << LOG2_ARRAY_INT_INDEX_SCALE - log2ArrayIndexScale;
            if (tail >= wordTail) {
                long bi = (long) wi << LOG2_ARRAY_LONG_INDEX_SCALE;
                int av = U.getIntUnaligned(a, aOffset + bi);
                int bv = U.getIntUnaligned(b, bOffset + bi);
                if (av != bv) {
                    return (wi << log2ValuesPerWidth) + (Integer.numberOfLeadingZeros(av ^ bv) >> LOG2_BYTE_BIT_SIZE + log2ArrayIndexScale);
                }
                tail -= wordTail;
            }
            return ~tail;
        } else {
            return ~tail;
        }
    }
    public static int hashCode(int[] a, int fromIndex, int length, int initialValue) {
        return switch (length) {
            case 0 -> initialValue;
            case 1 -> 31 * initialValue + a[fromIndex];
            default -> vectorizedHashCode(a, fromIndex, length, initialValue, T_INT);
        };
    }
    public static int hashCode(short[] a, int fromIndex, int length, int initialValue) {
        return switch (length) {
            case 0 -> initialValue;
            case 1 -> 31 * initialValue + a[fromIndex];
            default -> vectorizedHashCode(a, fromIndex, length, initialValue, T_SHORT);
        };
    }
    public static int hashCode(char[] a, int fromIndex, int length, int initialValue) {
        return switch (length) {
            case 0 -> initialValue;
            case 1 -> 31 * initialValue + a[fromIndex];
            default -> vectorizedHashCode(a, fromIndex, length, initialValue, T_CHAR);
        };
    }
    public static int hashCode(byte[] a, int fromIndex, int length, int initialValue) {
        return switch (length) {
            case 0 -> initialValue;
            case 1 -> 31 * initialValue + a[fromIndex];
            default -> vectorizedHashCode(a, fromIndex, length, initialValue, T_BYTE);
        };
    }
    public static int hashCodeOfUnsigned(byte[] a, int fromIndex, int length, int initialValue) {
        return switch (length) {
            case 0 -> initialValue;
            case 1 -> 31 * initialValue + Byte.toUnsignedInt(a[fromIndex]);
            default -> vectorizedHashCode(a, fromIndex, length, initialValue, T_BOOLEAN);
        };
    }
    public static int hashCodeOfUTF16(byte[] a, int fromIndex, int length, int initialValue) {
        return switch (length) {
            case 0 -> initialValue;
            case 1 -> 31 * initialValue + JLA.uncheckedGetUTF16Char(a, fromIndex);
            default -> vectorizedHashCode(a, fromIndex, length, initialValue, T_CHAR);
        };
    }
    public static int hashCode(Object[] a, int fromIndex, int length, int initialValue) {
        int result = initialValue;
        for (int i = fromIndex; i < fromIndex + length; i++) {
            result = 31 * result + Objects.hashCode(a[i]);
        }
        return result;
    }
    private static final int T_BOOLEAN = 4;
    private static final int T_CHAR = 5;
    private static final int T_FLOAT = 6;
    private static final int T_DOUBLE = 7;
    private static final int T_BYTE = 8;
    private static final int T_SHORT = 9;
    private static final int T_INT = 10;
    private static final int T_LONG = 11;
    @IntrinsicCandidate
    private static int vectorizedHashCode(Object array, int fromIndex, int length, int initialValue, int basicType) {
        return switch (basicType) {
            case T_BOOLEAN -> unsignedHashCode(initialValue, (byte[]) array, fromIndex, length);
            case T_CHAR -> array instanceof byte[] ? utf16hashCode(initialValue, (byte[]) array, fromIndex, length) : hashCode(initialValue, (char[]) array, fromIndex, length);
            case T_BYTE -> hashCode(initialValue, (byte[]) array, fromIndex, length);
            case T_SHORT -> hashCode(initialValue, (short[]) array, fromIndex, length);
            case T_INT -> hashCode(initialValue, (int[]) array, fromIndex, length);
            default -> throw new IllegalArgumentException("unrecognized basic type: " + basicType);
        };
    }
    private static int unsignedHashCode(int result, byte[] a, int fromIndex, int length) {
        for (int i = fromIndex; i < fromIndex + length; i++) {
            result = 31 * result + Byte.toUnsignedInt(a[i]);
        }
        return result;
    }
    private static int hashCode(int result, byte[] a, int fromIndex, int length) {
        for (int i = fromIndex; i < fromIndex + length; i++) {
            result = 31 * result + a[i];
        }
        return result;
    }
    private static int hashCode(int result, char[] a, int fromIndex, int length) {
        for (int i = fromIndex; i < fromIndex + length; i++) {
            result = 31 * result + a[i];
        }
        return result;
    }
    private static int hashCode(int result, short[] a, int fromIndex, int length) {
        for (int i = fromIndex; i < fromIndex + length; i++) {
            result = 31 * result + a[i];
        }
        return result;
    }
    private static int hashCode(int result, int[] a, int fromIndex, int length) {
        for (int i = fromIndex; i < fromIndex + length; i++) {
            result = 31 * result + a[i];
        }
        return result;
    }
    private static final JavaLangAccess JLA = SharedSecrets.getJavaLangAccess();
    private static int utf16hashCode(int result, byte[] value, int fromIndex, int length) {
        for (int i = fromIndex; i < fromIndex + length; i++) {
            result = 31 * result + JLA.uncheckedGetUTF16Char(value, i);
        }
        return result;
    }
    public static int mismatch(boolean[] a, boolean[] b, int length) {
        int i = 0;
        if (length > 7) {
            if (a[0] != b[0]) 
                return 0;
            i = vectorizedMismatch(a, Unsafe.ARRAY_BOOLEAN_BASE_OFFSET, b, Unsafe.ARRAY_BOOLEAN_BASE_OFFSET, length, LOG2_ARRAY_BOOLEAN_INDEX_SCALE);
            if (i >= 0) 
                return i;
            i = length - ~i;
        }
        for (; i < length; i++) {
            if (a[i] != b[i]) 
                return i;
        }
        return -1;
    }
    public static int mismatch(boolean[] a, int aFromIndex, boolean[] b, int bFromIndex, int length) {
        int i = 0;
        if (length > 7) {
            if (a[aFromIndex] != b[bFromIndex]) 
                return 0;
            i = vectorizedMismatch(a, Unsafe.ARRAY_BOOLEAN_BASE_OFFSET + aFromIndex, b, Unsafe.ARRAY_BOOLEAN_BASE_OFFSET + bFromIndex, length, LOG2_ARRAY_BOOLEAN_INDEX_SCALE);
            if (i >= 0) 
                return i;
            i = length - ~i;
        }
        for (; i < length; i++) {
            if (a[aFromIndex + i] != b[bFromIndex + i]) 
                return i;
        }
        return -1;
    }
    public static int mismatch(byte[] a, byte[] b, int length) {
        int i = 0;
        if (length > 7) {
            if (a[0] != b[0]) 
                return 0;
            i = vectorizedMismatch(a, Unsafe.ARRAY_BYTE_BASE_OFFSET, b, Unsafe.ARRAY_BYTE_BASE_OFFSET, length, LOG2_ARRAY_BYTE_INDEX_SCALE);
            if (i >= 0) 
                return i;
            i = length - ~i;
        }
        for (; i < length; i++) {
            if (a[i] != b[i]) 
                return i;
        }
        return -1;
    }
    public static int mismatch(byte[] a, int aFromIndex, byte[] b, int bFromIndex, int length) {
        int i = 0;
        if (length > 7) {
            if (a[aFromIndex] != b[bFromIndex]) 
                return 0;
            i = vectorizedMismatch(a, Unsafe.ARRAY_BYTE_BASE_OFFSET + aFromIndex, b, Unsafe.ARRAY_BYTE_BASE_OFFSET + bFromIndex, length, LOG2_ARRAY_BYTE_INDEX_SCALE);
            if (i >= 0) 
                return i;
            i = length - ~i;
        }
        for (; i < length; i++) {
            if (a[aFromIndex + i] != b[bFromIndex + i]) 
                return i;
        }
        return -1;
    }
    public static int mismatch(char[] a, char[] b, int length) {
        int i = 0;
        if (length > 3) {
            if (a[0] != b[0]) 
                return 0;
            i = vectorizedMismatch(a, Unsafe.ARRAY_CHAR_BASE_OFFSET, b, Unsafe.ARRAY_CHAR_BASE_OFFSET, length, LOG2_ARRAY_CHAR_INDEX_SCALE);
            if (i >= 0) 
                return i;
            i = length - ~i;
        }
        for (; i < length; i++) {
            if (a[i] != b[i]) 
                return i;
        }
        return -1;
    }
    public static int mismatch(char[] a, int aFromIndex, char[] b, int bFromIndex, int length) {
        int i = 0;
        if (length > 3) {
            if (a[aFromIndex] != b[bFromIndex]) 
                return 0;
            i = vectorizedMismatch(a, Unsafe.ARRAY_CHAR_BASE_OFFSET + (aFromIndex << LOG2_ARRAY_CHAR_INDEX_SCALE), b, Unsafe.ARRAY_CHAR_BASE_OFFSET + (bFromIndex << LOG2_ARRAY_CHAR_INDEX_SCALE), length, LOG2_ARRAY_CHAR_INDEX_SCALE);
            if (i >= 0) 
                return i;
            i = length - ~i;
        }
        for (; i < length; i++) {
            if (a[aFromIndex + i] != b[bFromIndex + i]) 
                return i;
        }
        return -1;
    }
    public static int mismatch(short[] a, short[] b, int length) {
        int i = 0;
        if (length > 3) {
            if (a[0] != b[0]) 
                return 0;
            i = vectorizedMismatch(a, Unsafe.ARRAY_SHORT_BASE_OFFSET, b, Unsafe.ARRAY_SHORT_BASE_OFFSET, length, LOG2_ARRAY_SHORT_INDEX_SCALE);
            if (i >= 0) 
                return i;
            i = length - ~i;
        }
        for (; i < length; i++) {
            if (a[i] != b[i]) 
                return i;
        }
        return -1;
    }
    public static int mismatch(short[] a, int aFromIndex, short[] b, int bFromIndex, int length) {
        int i = 0;
        if (length > 3) {
            if (a[aFromIndex] != b[bFromIndex]) 
                return 0;
            i = vectorizedMismatch(a, Unsafe.ARRAY_SHORT_BASE_OFFSET + (aFromIndex << LOG2_ARRAY_SHORT_INDEX_SCALE), b, Unsafe.ARRAY_SHORT_BASE_OFFSET + (bFromIndex << LOG2_ARRAY_SHORT_INDEX_SCALE), length, LOG2_ARRAY_SHORT_INDEX_SCALE);
            if (i >= 0) 
                return i;
            i = length - ~i;
        }
        for (; i < length; i++) {
            if (a[aFromIndex + i] != b[bFromIndex + i]) 
                return i;
        }
        return -1;
    }
    public static int mismatch(int[] a, int[] b, int length) {
        int i = 0;
        if (length > 1) {
            if (a[0] != b[0]) 
                return 0;
            i = vectorizedMismatch(a, Unsafe.ARRAY_INT_BASE_OFFSET, b, Unsafe.ARRAY_INT_BASE_OFFSET, length, LOG2_ARRAY_INT_INDEX_SCALE);
            if (i >= 0) 
                return i;
            i = length - ~i;
        }
        for (; i < length; i++) {
            if (a[i] != b[i]) 
                return i;
        }
        return -1;
    }
    public static int mismatch(int[] a, int aFromIndex, int[] b, int bFromIndex, int length) {
        int i = 0;
        if (length > 1) {
            if (a[aFromIndex] != b[bFromIndex]) 
                return 0;
            i = vectorizedMismatch(a, Unsafe.ARRAY_INT_BASE_OFFSET + (aFromIndex << LOG2_ARRAY_INT_INDEX_SCALE), b, Unsafe.ARRAY_INT_BASE_OFFSET + (bFromIndex << LOG2_ARRAY_INT_INDEX_SCALE), length, LOG2_ARRAY_INT_INDEX_SCALE);
            if (i >= 0) 
                return i;
            i = length - ~i;
        }
        for (; i < length; i++) {
            if (a[aFromIndex + i] != b[bFromIndex + i]) 
                return i;
        }
        return -1;
    }
    public static int mismatch(float[] a, float[] b, int length) {
        return mismatch(a, 0, b, 0, length);
    }
    public static int mismatch(float[] a, int aFromIndex, float[] b, int bFromIndex, int length) {
        int i = 0;
        if (length > 1) {
            if (Float.floatToRawIntBits(a[aFromIndex]) == Float.floatToRawIntBits(b[bFromIndex])) {
                i = vectorizedMismatch(a, Unsafe.ARRAY_FLOAT_BASE_OFFSET + (aFromIndex << LOG2_ARRAY_FLOAT_INDEX_SCALE), b, Unsafe.ARRAY_FLOAT_BASE_OFFSET + (bFromIndex << LOG2_ARRAY_FLOAT_INDEX_SCALE), length, LOG2_ARRAY_FLOAT_INDEX_SCALE);
            }
            if (i >= 0) {
                if (!Float.isNaN(a[aFromIndex + i]) || !Float.isNaN(b[bFromIndex + i])) 
                    return i;
                i++;
            } else {
                i = length - ~i;
            }
        }
        for (; i < length; i++) {
            if (Float.floatToIntBits(a[aFromIndex + i]) != Float.floatToIntBits(b[bFromIndex + i])) 
                return i;
        }
        return -1;
    }
    public static int mismatch(long[] a, long[] b, int length) {
        if (length == 0) {
            return -1;
        }
        if (a[0] != b[0]) 
            return 0;
        int i = vectorizedMismatch(a, Unsafe.ARRAY_LONG_BASE_OFFSET, b, Unsafe.ARRAY_LONG_BASE_OFFSET, length, LOG2_ARRAY_LONG_INDEX_SCALE);
        return i >= 0 ? i : -1;
    }
    public static int mismatch(long[] a, int aFromIndex, long[] b, int bFromIndex, int length) {
        if (length == 0) {
            return -1;
        }
        if (a[aFromIndex] != b[bFromIndex]) 
            return 0;
        int i = vectorizedMismatch(a, Unsafe.ARRAY_LONG_BASE_OFFSET + (aFromIndex << LOG2_ARRAY_LONG_INDEX_SCALE), b, Unsafe.ARRAY_LONG_BASE_OFFSET + (bFromIndex << LOG2_ARRAY_LONG_INDEX_SCALE), length, LOG2_ARRAY_LONG_INDEX_SCALE);
        return i >= 0 ? i : -1;
    }
    public static int mismatch(double[] a, double[] b, int length) {
        return mismatch(a, 0, b, 0, length);
    }
    public static int mismatch(double[] a, int aFromIndex, double[] b, int bFromIndex, int length) {
        if (length == 0) {
            return -1;
        }
        int i = 0;
        if (Double.doubleToRawLongBits(a[aFromIndex]) == Double.doubleToRawLongBits(b[bFromIndex])) {
            i = vectorizedMismatch(a, Unsafe.ARRAY_DOUBLE_BASE_OFFSET + (aFromIndex << LOG2_ARRAY_DOUBLE_INDEX_SCALE), b, Unsafe.ARRAY_DOUBLE_BASE_OFFSET + (bFromIndex << LOG2_ARRAY_DOUBLE_INDEX_SCALE), length, LOG2_ARRAY_DOUBLE_INDEX_SCALE);
        }
        if (i >= 0) {
            if (!Double.isNaN(a[aFromIndex + i]) || !Double.isNaN(b[bFromIndex + i])) 
                return i;
            i++;
            for (; i < length; i++) {
                if (Double.doubleToLongBits(a[aFromIndex + i]) != Double.doubleToLongBits(b[bFromIndex + i])) 
                    return i;
            }
        }
        return -1;
    }
    public static final int SOFT_MAX_ARRAY_LENGTH = Integer.MAX_VALUE - 8;
    public static int newLength(int oldLength, int minGrowth, int prefGrowth) {
        int prefLength = oldLength + Math.max(minGrowth, prefGrowth);
        return 0 < prefLength && prefLength <= SOFT_MAX_ARRAY_LENGTH ? prefLength : hugeLength(oldLength, minGrowth);
    }
    private static int hugeLength(int oldLength, int minGrowth) {
        int minLength = oldLength + minGrowth;
        if (minLength < 0) {
            throw new OutOfMemoryError("Required array length " + oldLength + " + " + minGrowth + " is too large");
        } else 
            return minLength <= SOFT_MAX_ARRAY_LENGTH ? SOFT_MAX_ARRAY_LENGTH : minLength;
    }
    public static <T> T[] reverse(T[] a) {
        int limit = a.length / 2;
        for (int i = 0, j = a.length - 1; i < limit; i++, j--) {
            T t = a[i];
            a[i] = a[j];
            a[j] = t;
        }
        return a;
    }
    public static <T> T[] toArrayReversed(Collection<?> coll, T[] array) {
        T[] newArray = reverse(coll.toArray(Arrays.copyOfRange(array, 0, 0)));
        if (newArray.length > array.length) {
            return newArray;
        } else {
            System.arraycopy(newArray, 0, array, 0, newArray.length);
            if (array.length > newArray.length) {
                array[newArray.length] = null;
            }
            return array;
        }
    }
}
