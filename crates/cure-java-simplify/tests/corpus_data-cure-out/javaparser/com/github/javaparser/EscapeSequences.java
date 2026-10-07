package com.github.javaparser;

public class EscapeSequences {
    public static void main(String[] args) {
        Object[] chars = {'\\', '\\', '\\', "---", '\n', '\n', '\n', "---", '\r', '\r', '\r', "---", '\t', '\t', '\t', "---", '\u0008', '\u0008', '\u0008', "---", '\u000C', '\u000C', '\u000C', "---", '\'', '\'', "---", '"', '"', '"'};
        for (Object obj : chars) {
            if (obj instanceof Character) {
                System.out.println(obj + " " + (int) (char) obj);
            } else {
                System.out.println(obj);
            }
        }
    }
}
