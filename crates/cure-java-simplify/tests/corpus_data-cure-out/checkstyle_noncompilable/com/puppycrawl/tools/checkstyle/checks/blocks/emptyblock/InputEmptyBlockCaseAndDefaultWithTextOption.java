package com.puppycrawl.tools.checkstyle.checks.blocks.emptyblock;

public class InputEmptyBlockCaseAndDefaultWithTextOption {
    void testWithEmptyBlocks(Object obj) {
        switch (obj) {
            case Integer i:
                {
                    System.out.println("Integer");
                }
            case String _:
                {}
            default:
                {}
        }
        switch (obj) {
            case Integer i -> {
                System.out.println("Integer");
            }
            case String s -> {}
            default -> {}
        }
        switch (obj) {
            case Integer i:
                {
                    System.out.println("Integer");
                }
            case String _:
                {
                    System.out.println("String");
                }
            default:
                {
                    System.out.println("defuault");
                }
        }
        switch (obj) {
            case Integer i -> {
                System.out.println("Integer");
            }
            case String s -> {
                System.out.println("String");
            }
            default -> {
                System.out.println("defuault");
            }
        }
    }
    void testWithTextInsideBlocks(Object obj) {
        switch (obj) {
            case Integer i:
                {
                    System.out.println("Integer");
                }
            case String _:
                {}
            default:
                {}
        }
        switch (obj) {
            case Integer i -> {
                System.out.println("Integer");
            }
            case String s -> {}
            default -> {}
        }
    }
}
