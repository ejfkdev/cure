package java.util;

import java.io.IOException;
import java.lang.invoke.*;
import java.lang.invoke.MethodHandles.Lookup;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.text.DecimalFormatSymbols;
import java.util.Formatter.FormatSpecifier;
import jdk.internal.access.JavaLangAccess;
import jdk.internal.access.SharedSecrets;
import jdk.internal.util.FormatConcatItem;
import static java.lang.invoke.MethodType.methodType;

class FormatItem {
    private static final JavaLangAccess JLA = SharedSecrets.getJavaLangAccess();
    private static final MethodHandle CHAR_MIX = JLA.stringConcatHelper("mix", MethodType.methodType(long.class, long.class, char.class));
    private static final MethodHandle STRING_PREPEND = JLA.stringConcatHelper("prepend", MethodType.methodType(long.class, long.class, byte[].class, String.class, String.class));
    private static final MethodHandle SELECT_GETCHAR_MH = JLA.stringConcatHelper("selectGetChar", MethodType.methodType(MethodHandle.class, long.class));
    private static final MethodHandle SELECT_PUTCHAR_MH = JLA.stringConcatHelper("selectPutChar", MethodType.methodType(MethodHandle.class, long.class));
    private static long charMix(long lengthCoder, char value) {
        try {
            return (long) CHAR_MIX.invokeExact(lengthCoder, value);
        } catch (Throwable ex) {
            throw new RuntimeException(ex);
        }
    }
    private static long stringMix(long lengthCoder, String value) {
        return JLA.stringConcatMix(lengthCoder, value);
    }
    private static long stringPrepend(long lengthCoder, byte[] buffer, String value) throws Throwable {
        return (long) STRING_PREPEND.invokeExact(lengthCoder, buffer, value, (String) null);
    }
    private static MethodHandle selectGetChar(long indexCoder) throws Throwable {
        return (MethodHandle) SELECT_GETCHAR_MH.invokeExact(indexCoder);
    }
    private static MethodHandle selectPutChar(long indexCoder) throws Throwable {
        return (MethodHandle) SELECT_PUTCHAR_MH.invokeExact(indexCoder);
    }
    private static final MethodHandle PUT_CHAR_DIGIT;
    static {
        try {
            PUT_CHAR_DIGIT = MethodHandles.lookup().findStatic(FormatItem.class, "putByte", MethodType.methodType(void.class, byte[].class, int.class, int.class));
        } catch (ReflectiveOperationException ex) {
            throw new AssertionError("putByte lookup failed", ex);
        }
    }
    private static void putByte(byte[] buffer, int index, int ch) {
        buffer[index] = (byte) ch;
    }
    private FormatItem() {
        throw new AssertionError("private constructor");
    }
    static final class FormatItemDecimal implements FormatConcatItem {
        private final char groupingSeparator;
        private final char zeroDigit;
        private final char minusSign;
        private final int digitOffset;
        private final byte[] digits;
        private final int length;
        private final boolean isNegative;
        private final int width;
        private final byte prefixSign;
        private final int groupSize;
        private final long value;
        private final boolean parentheses;
        FormatItemDecimal(DecimalFormatSymbols dfs, int width, char sign, boolean parentheses, int groupSize, long value) throws Throwable {
            this.groupingSeparator = dfs.getGroupingSeparator();
            this.zeroDigit = dfs.getZeroDigit();
            this.minusSign = dfs.getMinusSign();
            this.digitOffset = this.zeroDigit - '0';
            int length = DecimalDigits.INSTANCE.size(value);
            this.digits = new byte[length];
            DecimalDigits.INSTANCE.digits(value, this.digits, length, PUT_CHAR_DIGIT);
            this.isNegative = value < 0L;
            this.length = this.isNegative ? length - 1 : length;
            this.width = width;
            this.groupSize = groupSize;
            this.value = value;
            this.parentheses = parentheses && isNegative;
            this.prefixSign = (byte) (isNegative ? parentheses ? '\u0000' : minusSign : sign);
        }
        private int signLength() {
            return (prefixSign != '\u0000' ? 1 : 0) + (parentheses ? 2 : 0);
        }
        private int groupLength() {
            return 0 < groupSize ? (length - 1) / groupSize : 0;
        }
        @Override
        public long mix(long lengthCoder) {
            return JLA.stringConcatCoder(zeroDigit) | lengthCoder + Integer.max(length + signLength() + groupLength(), width);
        }
        @Override
        public long prepend(long lengthCoder, byte[] buffer) throws Throwable {
            MethodHandle putCharMH = selectPutChar(lengthCoder);
            if (parentheses) {
                putCharMH.invokeExact(buffer, (int) --lengthCoder, 41);
            }
            if (0 < groupSize) {
                int groupIndex = groupSize;
                for (int i = 1; i <= length; i++) {
                    if (groupIndex-- == 0) {
                        putCharMH.invokeExact(buffer, (int) --lengthCoder, (int) groupingSeparator);
                        groupIndex = groupSize - 1;
                    }
                    putCharMH.invokeExact(buffer, (int) --lengthCoder, digits[digits.length - i] + digitOffset);
                }
            } else {
                for (int i = 1; i <= length; i++) {
                    putCharMH.invokeExact(buffer, (int) --lengthCoder, digits[digits.length - i] + digitOffset);
                }
            }
            for (int i = length + signLength() + groupLength(); i < width; i++) {
                putCharMH.invokeExact(buffer, (int) --lengthCoder, 48);
            }
            if (parentheses) {
                putCharMH.invokeExact(buffer, (int) --lengthCoder, 40);
            }
            if (prefixSign != '\u0000') {
                putCharMH.invokeExact(buffer, (int) --lengthCoder, (int) prefixSign);
            }
            return lengthCoder;
        }
    }
    static final class FormatItemHexadecimal implements FormatConcatItem {
        private final int width;
        private final boolean hasPrefix;
        private final long value;
        private final int length;
        FormatItemHexadecimal(int width, boolean hasPrefix, long value) {
            this.width = width;
            this.hasPrefix = hasPrefix;
            this.value = value;
            this.length = HexDigits.INSTANCE.size(value);
        }
        private int prefixLength() {
            return hasPrefix ? 2 : 0;
        }
        private int zeroesLength() {
            return Integer.max(0, width - length - prefixLength());
        }
        @Override
        public long mix(long lengthCoder) {
            return lengthCoder + length + prefixLength() + zeroesLength();
        }
        @Override
        public long prepend(long lengthCoder, byte[] buffer) throws Throwable {
            MethodHandle putCharMH = selectPutChar(lengthCoder);
            HexDigits.INSTANCE.digits(value, buffer, (int) lengthCoder, putCharMH);
            lengthCoder -= length;
            for (int i = 0; i < zeroesLength(); i++) {
                putCharMH.invokeExact(buffer, (int) --lengthCoder, 48);
            }
            if (hasPrefix) {
                putCharMH.invokeExact(buffer, (int) --lengthCoder, 120);
                putCharMH.invokeExact(buffer, (int) --lengthCoder, 48);
            }
            return lengthCoder;
        }
    }
    static final class FormatItemOctal implements FormatConcatItem {
        private final int width;
        private final boolean hasPrefix;
        private final long value;
        private final int length;
        FormatItemOctal(int width, boolean hasPrefix, long value) {
            this.width = width;
            this.hasPrefix = hasPrefix;
            this.value = value;
            this.length = OctalDigits.INSTANCE.size(value);
        }
        private int prefixLength() {
            return hasPrefix && value != 0 ? 1 : 0;
        }
        private int zeroesLength() {
            return Integer.max(0, width - length - prefixLength());
        }
        @Override
        public long mix(long lengthCoder) {
            return lengthCoder + length + prefixLength() + zeroesLength();
        }
        @Override
        public long prepend(long lengthCoder, byte[] buffer) throws Throwable {
            MethodHandle putCharMH = selectPutChar(lengthCoder);
            OctalDigits.INSTANCE.digits(value, buffer, (int) lengthCoder, putCharMH);
            lengthCoder -= length;
            for (int i = 0; i < zeroesLength(); i++) {
                putCharMH.invokeExact(buffer, (int) --lengthCoder, 48);
            }
            if (hasPrefix && value != 0) {
                putCharMH.invokeExact(buffer, (int) --lengthCoder, 48);
            }
            return lengthCoder;
        }
    }
    static final class FormatItemBoolean implements FormatConcatItem {
        private final boolean value;
        FormatItemBoolean(boolean value) {
            this.value = value;
        }
        @Override
        public long mix(long lengthCoder) {
            return lengthCoder + (value ? 4 : 5);
        }
        @Override
        public long prepend(long lengthCoder, byte[] buffer) throws Throwable {
            MethodHandle putCharMH = selectPutChar(lengthCoder);
            if (value) {
                putCharMH.invokeExact(buffer, (int) --lengthCoder, 101);
                putCharMH.invokeExact(buffer, (int) --lengthCoder, 117);
                putCharMH.invokeExact(buffer, (int) --lengthCoder, 114);
                putCharMH.invokeExact(buffer, (int) --lengthCoder, 116);
            } else {
                putCharMH.invokeExact(buffer, (int) --lengthCoder, 101);
                putCharMH.invokeExact(buffer, (int) --lengthCoder, 115);
                putCharMH.invokeExact(buffer, (int) --lengthCoder, 108);
                putCharMH.invokeExact(buffer, (int) --lengthCoder, 97);
                putCharMH.invokeExact(buffer, (int) --lengthCoder, 102);
            }
            return lengthCoder;
        }
    }
    static final class FormatItemCharacter implements FormatConcatItem {
        private final char value;
        FormatItemCharacter(char value) {
            this.value = value;
        }
        @Override
        public long mix(long lengthCoder) {
            return charMix(lengthCoder, value);
        }
        @Override
        public long prepend(long lengthCoder, byte[] buffer) throws Throwable {
            selectPutChar(lengthCoder).invokeExact(buffer, (int) --lengthCoder, (int) value);
            return lengthCoder;
        }
    }
    static final class FormatItemString implements FormatConcatItem {
        private String value;
        FormatItemString(String value) {
            this.value = value;
        }
        @Override
        public long mix(long lengthCoder) {
            return stringMix(lengthCoder, value);
        }
        @Override
        public long prepend(long lengthCoder, byte[] buffer) throws Throwable {
            return stringPrepend(lengthCoder, buffer, value);
        }
    }
    static final class FormatItemFormatSpecifier implements FormatConcatItem {
        private StringBuilder sb;
        FormatItemFormatSpecifier(FormatSpecifier fs, Locale locale, Object value) {
            this.sb = new StringBuilder(64);
            Formatter formatter = new Formatter(this.sb, locale);
            try {
                fs.print(formatter, value, locale);
            } catch (IOException ex) {
                throw new AssertionError("FormatItemFormatSpecifier IOException", ex);
            }
        }
        FormatItemFormatSpecifier(Locale locale, int flags, int width, int precision, Formattable formattable) {
            this.sb = new StringBuilder(64);
            Formatter formatter = new Formatter(this.sb, locale);
            formattable.formatTo(formatter, flags, width, precision);
        }
        @Override
        public long mix(long lengthCoder) {
            return JLA.stringBuilderConcatMix(lengthCoder, sb);
        }
        @Override
        public long prepend(long lengthCoder, byte[] buffer) throws Throwable {
            return JLA.stringBuilderConcatPrepend(lengthCoder, buffer, sb);
        }
    }
    static abstract sealed class FormatItemModifier permits FormatItemFillLeft, FormatItemFillRight implements FormatConcatItem {
        private final long itemLengthCoder;
        protected final FormatConcatItem item;
        FormatItemModifier(FormatConcatItem item) {
            this.itemLengthCoder = item.mix(0L);
            this.item = item;
        }
        int length() {
            return (int) itemLengthCoder;
        }
        long coder() {
            return itemLengthCoder & ~Integer.MAX_VALUE;
        }
        @Override
        public abstract long mix(long lengthCoder);
        @Override
        public abstract long prepend(long lengthCoder, byte[] buffer) throws Throwable;
    }
    static final class FormatItemFillLeft extends FormatItemModifier implements FormatConcatItem {
        private final int width;
        FormatItemFillLeft(int width, FormatConcatItem item) {
            super(item);
            this.width = Integer.max(length(), width);
        }
        @Override
        public long mix(long lengthCoder) {
            return (lengthCoder | coder()) + width;
        }
        @Override
        public long prepend(long lengthCoder, byte[] buffer) throws Throwable {
            MethodHandle putCharMH = selectPutChar(lengthCoder);
            lengthCoder = item.prepend(lengthCoder, buffer);
            for (int i = length(); i < width; i++) {
                putCharMH.invokeExact(buffer, (int) --lengthCoder, 32);
            }
            return lengthCoder;
        }
    }
    static final class FormatItemFillRight extends FormatItemModifier implements FormatConcatItem {
        private final int width;
        FormatItemFillRight(int width, FormatConcatItem item) {
            super(item);
            this.width = Integer.max(length(), width);
        }
        @Override
        public long mix(long lengthCoder) {
            return (lengthCoder | coder()) + width;
        }
        @Override
        public long prepend(long lengthCoder, byte[] buffer) throws Throwable {
            MethodHandle putCharMH = selectPutChar(lengthCoder);
            for (int i = length(); i < width; i++) {
                putCharMH.invokeExact(buffer, (int) --lengthCoder, 32);
            }
            lengthCoder = item.prepend(lengthCoder, buffer);
            return lengthCoder;
        }
    }
    static final class FormatItemNull implements FormatConcatItem {
        FormatItemNull() {}
        @Override
        public long mix(long lengthCoder) {
            return lengthCoder + 4;
        }
        @Override
        public long prepend(long lengthCoder, byte[] buffer) throws Throwable {
            MethodHandle putCharMH = selectPutChar(lengthCoder);
            putCharMH.invokeExact(buffer, (int) --lengthCoder, 108);
            putCharMH.invokeExact(buffer, (int) --lengthCoder, 108);
            putCharMH.invokeExact(buffer, (int) --lengthCoder, 117);
            putCharMH.invokeExact(buffer, (int) --lengthCoder, 110);
            return lengthCoder;
        }
    }
}
