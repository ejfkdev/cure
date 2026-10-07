package japa.bdd.samples;

import com.github.javaparser.JavaParser;
import japa.parser.ParseException;
import com.github.javaparser.ast.CompilationUnit;
import java.io.*;
import java.util.*;

@Deprecated
public class JavaConceptsBase<T extends List<int[]>, X> extends Base implements Serializable {
    static Class clz1 = String.class;
    protected Class clz2 = String.class;
    Class clz3 = int.class;
    Class clz4 = int.class;
    int[] arr = new int[10];
    byte bye = 0;
    byte[] byebye = null;
    short sh1, sh2 = 1;
    int intWithUnderscore = 1234_5678;
    long longWithUnderscore = 1234_5678L;
    float floatWithUnderscore = 1_234.5_678f;
    float floatWithUnderscoreAndExponent = 1_234e1_0f;
    double doubleWithUnderscore = 1_234.5_678;
    double doubleWithUnderscoreAndExponent = 1_234e1_0;
    int binaryLiteral = 0b101101;
    List<String>[][] arrLS = (List<String>[][]) new List<?>[10][];
    int[][][][] arr2 = new int[10][2][1][0];
    volatile float fff = 0x1.fffeP+127f;
    char cc = 'a';
    int[][] arr3 = {{1, 2}, {3, 4}};
    static int[][] arr4 = {};
    static {
        arr4 = new int[][] {{2}, {1}};
    }
    {
        arr3 = new int[][] {{2}, {1}};
    }
    public static JavaConceptsBase t;
}
