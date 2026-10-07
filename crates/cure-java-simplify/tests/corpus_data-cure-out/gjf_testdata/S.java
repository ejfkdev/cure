package com.google.googlejavaformat.java.test;

class S {
    int x = 0;
    @SingleMemberAnnotation(
      0 + 0 + 0 + 0 + 0 + 0 + 0 + 0 + 0 + 0 + 0 + 0 + 0 + 0 + 0 + 0 + 0 + 0 + 0 + 0 + 0 + 0 + 0 + 0
          + 0 + 0 + 0 + 0 + 0 + 0 + 0 + 0) S() {
        super();
    }
    class SS extends S {
        SS() {
            super();
            super.x = 0;
            super.foo();
        }
    }
    void foo() {
        synchronized (null[0]) {
            switch ("abc") {
                case "one":
                    break;
                case "two":
                    break;
                case "three":
                default:
                    break;
            }
        }
    }
}
