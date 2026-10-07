package com.google.googlejavaformat.java;

import static com.google.common.truth.Truth.assertThat;
import static com.google.googlejavaformat.java.RemoveUnusedImports.removeUnusedImports;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

@RunWith(JUnit4.class)
public class RemoveUnusedImportsCaseLabelsTest {
    @Test
  public void preserveTypesInCaseLabels() throws FormatterException {
        String input = """
        package example;
        import example.model.SealedInterface;
        import example.model.TypeA;
        import example.model.TypeB;
        public class Main {
          public void apply(SealedInterface sealedInterface) {
            switch(sealedInterface) {
              case TypeA a -> System.out.println("A!");
              case TypeB b -> System.out.println("B!");
            }
          }
        }\
        """;
        assertThat(removeUnusedImports(input)).isEqualTo(input);
    }
}
