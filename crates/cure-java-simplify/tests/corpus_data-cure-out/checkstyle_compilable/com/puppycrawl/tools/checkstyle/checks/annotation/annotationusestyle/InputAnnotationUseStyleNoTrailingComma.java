package com.puppycrawl.tools.checkstyle.checks.annotation.annotationusestyle;

@SuppressWarnings({})
public class InputAnnotationUseStyleNoTrailingComma {
    @SuppressWarnings({"common"})
  public void foo() {
        @SuppressWarnings({"common","foo"})
              Object o = new Object() {

                  // violation below 'Annotation array values must contain trailing comma'
                  @SuppressWarnings(value ={"common"})
                  public String toString() {

                      // violation below 'Annotation array values must contain trailing comma'
                      @SuppressWarnings( value={"leo","herbie"})
                      final String pooches = "leo.herbie";

                      return pooches;
                  }
              };
    }
    @Test2(value={"foo"}, more={"bar"})
  @Pooches2(tokens={},other={}) enum P {
        @Pooches2(tokens={Pooches2.class},other={1})
      L, @Test2(value={}, more={"unchecked"})
      Y
    }
}

@interface Test2 {
    String[] value();
    String[] more() default {};
}

@interface Pooches2 {
    Class<?>[] tokens();
    int[] other();
}
