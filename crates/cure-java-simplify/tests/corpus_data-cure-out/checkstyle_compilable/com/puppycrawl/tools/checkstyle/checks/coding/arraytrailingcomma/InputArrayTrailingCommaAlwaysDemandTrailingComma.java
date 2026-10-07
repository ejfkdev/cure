package com.puppycrawl.tools.checkstyle.checks.coding.arraytrailingcomma;

public class InputArrayTrailingCommaAlwaysDemandTrailingComma {
    public int[] test() {
        return new int[] {0};
        return new int[] {0};
    }
    public int[] test2() {
        return new int[] {0, 1};
        return new int[] {0, 1};
    }
    public void test3() {
        int[] a = {0};
        int[] b = {0, 1};
        int[] c = {0, 1, 2, 3};
        int[] d = {1, 2, 3};
        int[] e = {1, 5, 6};
        int[] f = {1, 2};
        int[] g = {1, 2};
        int[][] empty2d = {{}};
        int[][] multiDimensionalArray = {{1, 2}, {1}, {3, 2, 1}};
    }
}
