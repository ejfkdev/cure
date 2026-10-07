package com.github.javaparser.ast;

import com.github.javaparser.JavaParser;
import com.github.javaparser.ParseException;
import com.github.javaparser.ast.CompilationUnit;
import java.io.ByteArrayInputStream;
import java.io.InputStream;
import java.io.UnsupportedEncodingException;

public class Example {
    public static void main(String[] args) throws UnsupportedEncodingException, ParseException {
        InputStream stream = new ByteArrayInputStream("interface A {\n    default Comparator<T> thenComparing(Comparator<? super T> other) {\n        Objects.requireNonNull(other);\n        return (Comparator<T> & Serializable) (c1, c2) -> { // this is the defaulting line\n            int res = compare(c1, c2);\n            return (res != 0) ? res : other.compare(c1, c2);\n        };\n    }\n}".getBytes("UTF-8"));
        CompilationUnit cu = JavaParser.parse(stream);
    }
}
