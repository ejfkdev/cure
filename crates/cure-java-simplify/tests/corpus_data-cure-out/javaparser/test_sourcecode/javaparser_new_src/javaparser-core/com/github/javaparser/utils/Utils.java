package com.github.javaparser.utils;

import com.github.javaparser.Provider;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.Reader;
import java.util.*;

public class Utils {
    public static final String EOL = System.getProperty("line.separator");
    public static <T> List<T> ensureNotNull(List<T> list) {
        return list == null ? new ArrayList<T>() : list;
    }
    public static <E> boolean isNullOrEmpty(Collection<E> collection) {
        return collection == null || collection.isEmpty();
    }
    public static <T> T assertNotNull(T o) {
        if (o == null) {
            throw new NullPointerException("Assertion failed.");
        }
        return o;
    }
    public static String escapeEndOfLines(String string) {
        StringBuilder escapedString = new StringBuilder();
        for (char c : string.toCharArray()) {
            switch (c) {
                case '\n':
                    escapedString.append("\\n");
                    break;
                case '\r':
                    escapedString.append("\\r");
                    break;
                default:
                    escapedString.append(c);
            }
        }
        return escapedString.toString();
    }
    public static String readerToString(Reader reader) throws IOException {
        StringBuilder result = new StringBuilder();
        char[] buffer = new char[8192];
        int numChars;
        while ((numChars = reader.read(buffer, 0, buffer.length)) > 0) {
            result.append(buffer, 0, numChars);
        }
        return result.toString();
    }
    public static String providerToString(Provider provider) throws IOException {
        StringBuilder result = new StringBuilder();
        char[] buffer = new char[8192];
        int numChars;
        while ((numChars = provider.read(buffer, 0, buffer.length)) != -1) {
            result.append(buffer, 0, numChars);
        }
        return result.toString();
    }
    public static <T> List<T> arrayToList(T[] array) {
        List<T> list = new LinkedList<>();
        Collections.addAll(list, array);
        return list;
    }
}
