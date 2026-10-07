package com.google.googlejavaformat.java;

import static com.google.common.truth.Truth.assertThat;
import com.google.common.base.Joiner;
import com.google.common.base.Splitter;
import com.google.common.collect.Iterables;
import java.util.Arrays;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.Parameterized;
import org.junit.runners.Parameterized.Parameters;

@RunWith(Parameterized.class)
public class ArrayDimensionTest {
    @Parameters
  public static Iterable<Object[]> parameters() {
        String[] inputs = {"int a[], b @B [], c @B [][] @C [];", "int a @A [], b @B [], c @B [] @C [];", "int a[] @A [], b @B [], c @B [] @C [];", "int a, b @B [], c @B [] @C [];", "int @A [] a, b @B [], c @B [] @C [];", "int @A [] a = {}, b @B [] = {{}}, c @B [] @C [] = {{{}}};", "int[][][][][] argh()[] @A @B [] @C @D [][] {}", "int[][] @T [] @U [] @V @W [][][] argh() @A @B [] @C @D [] {}", "int e1() @A [] {}", "int f1()[] @A [] {}", "int g1() @A [] @B [] {}", "int h1() @A @B [] @C @B [] {}", "int[] e2() @A [] {}", "int @X [] f2()[] @A [] {}", "int[] g2() @A [] @B [] {}", "int @X [] h2() @A @B [] @C @B [] {}", "@X int[] e3() @A [] {}", "@X int @Y [] f3()[] @A [] {}", "@X int @Y [] g3() @A [] @B [] {}", "@X int[] h3() @A @B [] @C @B [] {}", "int[] e2() @A [] {}", "int @I [] f2()[] @A [] {}", "int[] @J [] g2() @A [] @B [] {}", "int @I [] @J [] h2() @A @B [] @C @B [] {}", "int a1[];", "int b1 @A [];", "int c1[] @A [];", "int d1 @A [] @B [];", "int[] a2[];", "int @A [] b2 @B [];", "int[] c2[] @A [];", "int @A [] d2 @B [] @C [];", "int[][] a0 = new @A int[0];", "int[][] a1 = new int @A [0] @B [];", "int[][] a2 = new int[0] @A [] @B [];", "int[][] a4 = new int[0] @A [][] @B [];", "List<int @A [] @B []> xs;", "List<int[] @A [][] @B []> xs;"};
        return Iterables.transform(Arrays.asList(inputs), (input) -> new Object[] {input});
    }
    private final String input;
    public ArrayDimensionTest(String input) {
        this.input = input;
    }
    @Test
  public void format() throws Exception {
        String formatted = new Formatter().formatSource("class T {" + input + "}");
        String statement = formatted.substring(9, formatted.length() - 2);
        statement = Joiner.on(' ').join(Splitter.on('\n').omitEmptyStrings().trimResults().split(statement));
        assertThat(statement).isEqualTo(input);
    }
}
