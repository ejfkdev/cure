package com.puppycrawl.tools.checkstyle.grammar;

public class InputJava7StringSwitch {
    public static void main(String[] args) {
        switch ("value2") {
            case "value1":
                break;
            case "value2":
                break;
            default:
                break;
        }
    }
}
