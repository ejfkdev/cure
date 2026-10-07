package spoon.test.variable.testclasses.digest;

import java.io.IOException;
import java.io.InputStream;

public class DigestUtil {
    private static final int STREAM_BUFFER_LENGTH = 1024;
    public static MessageDigest getDigest(final String algorithm) {
        return new MessageDigest();
    }
    public static MessageDigest getMd2Digest() {
        return getDigest(MessageDigest.MD2);
    }
    public static MessageDigest getMd5Digest() {
        return getDigest(MessageDigest.MD5);
    }
    public static byte[] digest(final java.security.MessageDigest messageDigest, final byte[] data) {
        return messageDigest.digest(data);
    }
    public static byte[] digest(final java.security.MessageDigest messageDigest, final java.nio.ByteBuffer data) {
        messageDigest.update(data);
        return messageDigest.digest();
    }
    public static MessageDigest updateDigest(final MessageDigest digest, final InputStream data) throws IOException {
        byte[] buffer = new byte[STREAM_BUFFER_LENGTH];
        int read = data.read(buffer, 0, STREAM_BUFFER_LENGTH);
        while (read > -1) {
            read = data.read(buffer, 0, STREAM_BUFFER_LENGTH);
        }
        return digest;
    }
}
