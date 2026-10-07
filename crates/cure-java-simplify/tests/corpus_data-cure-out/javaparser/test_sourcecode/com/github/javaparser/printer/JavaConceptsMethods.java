package japa.bdd.samples;

import com.github.javaparser.JavaParser;
import japa.parser.ParseException;
import com.github.javaparser.ast.CompilationUnit;
import java.io.*;
import java.util.*;

public class JavaConceptsMethods {
    strictfp double ddd() {
        return 0.0;
    }
    public static void main(String[] args) throws ParseException, IOException {
        int x = 1;
        CompilationUnit cu = parse(new File("src/japa/parser/javacc/Parser.java"));
        System.out.println(cu);
        JavaConcepts teste = new JavaConcepts(2);
        JavaConcepts.QWE qwe = teste.new QWE(1);
        teste = new JavaConcepts(1);
        while (true) {
            xxx:
                while (x == 3) 
                    continue xxx;
            break;
        }
        do {
            x++;
        } while (x < 100);
        do 
            x++; while (x < 100);
        for (int i : arr4[0]) {
            x--;
        }
        for (int i = 0, j = 1; i < 10; x++) {
            break;
        }
        int i;
        int j;
        for (i = 0, j = 1; i < 10 && j < 2; i++, j--) {
            break;
        }
    }
    public static CompilationUnit parse(@Deprecated File file) throws ParseException, IOException {
        String a = (String) "qwe";
        int y = ((Integer) (Object) (String) clz1.getName()).intValue();
        synchronized (file) {
            file = new File("");
        }
        try {
            if (file == null) {
                throw new NullPointerException("blah");
            }
        } catch (NullPointerException e) {
            System.out.println("catch");
        } catch (RuntimeException e) {
            System.out.println("catch");
        } finally {
            System.out.println("finally");
        }
        try {
            if (file == null) {
                throw new NullPointerException("blah");
            }
        } finally {
            System.out.println("finally");
        }
        try {
            if (file == null) {
                throw new NullPointerException("blah");
            }
        } catch (RuntimeException e) {
            System.out.println("catch");
        }
        try (InputStream in = createInputStream()) {
            System.out.println(in);
        } catch (IOException e) {
            System.out.println("catch");
        }
        try (InputStream in = createInputStream(); InputStream in2 = createInputStream()) {
            System.out.println(in);
        } catch (IOException e) {
            System.out.println("catch");
        }
        try (InputStream in = createInputStream()) {
            System.out.println(in);
        }
        try {
            System.out.println("whatever");
        } catch (RuntimeException e) {
            System.out.println(e);
        } catch (Exception | Error e) {
            System.out.println(e);
        }
        return JavaParser.parse(file);
    }
    class A<T extends Integer & Serializable> implements XXX, Serializable {
        public <ABC> A(Integer integer, ABC string) throws Exception, IOException {}
    }
    private <Y> void x(Map<? extends X, ? super T> x) {
        @Deprecated Comparator c = new Comparator() {

                    public int compare(Object o1, Object o2) {
                        try {
                            A<Integer> a = new <String> A<Integer>(new Integer(11), "foo") {
                            };
                        } catch (Exception e) {
                        }
                        return 0;
                    }

                    @Override
                    public boolean equals(Object obj) {
                        return super.equals(obj);
                    }
                };
    }
    private static InputStream createInputStream() {
        return new ByteArrayInputStream(null);
    }
}
