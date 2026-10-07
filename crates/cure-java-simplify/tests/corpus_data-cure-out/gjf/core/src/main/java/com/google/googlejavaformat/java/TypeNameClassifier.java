package com.google.googlejavaformat.java;

import com.google.common.base.Verify;
import java.util.List;
import java.util.Optional;

final class TypeNameClassifier {
    private TypeNameClassifier() {}
    private enum TyParseState {
        START(false) {
      @Override
      public TyParseState next(JavaCaseFormat n) {
        return switch (n) {
          case UPPERCASE ->
              // if we see an UpperCamel later, assume this was a class
              // e.g. com.google.FOO.Bar
              TyParseState.AMBIGUOUS;
          case LOWER_CAMEL -> TyParseState.REJECT;
          case LOWERCASE ->
              // could be a package
              TyParseState.START;
          case UPPER_CAMEL -> TyParseState.TYPE;
        };
      }
    }, TYPE(true) {
      @Override
      public TyParseState next(JavaCaseFormat n) {
        return switch (n) {
          case UPPERCASE, LOWER_CAMEL, LOWERCASE -> TyParseState.FIRST_STATIC_MEMBER;
          case UPPER_CAMEL -> TyParseState.TYPE;
        };
      }
    }, FIRST_STATIC_MEMBER(true) {
      @Override
      public TyParseState next(JavaCaseFormat n) {
        return TyParseState.REJECT;
      }
    }, REJECT(false) {
      @Override
      public TyParseState next(JavaCaseFormat n) {
        return TyParseState.REJECT;
      }
    }, AMBIGUOUS(false) {
      @Override
      public TyParseState next(JavaCaseFormat n) {
        return switch (n) {
          case UPPERCASE -> AMBIGUOUS;
          case LOWER_CAMEL, LOWERCASE -> TyParseState.REJECT;
          case UPPER_CAMEL -> TyParseState.TYPE;
        };
      }
    };
        private final boolean isSingleUnit;
        TyParseState(boolean isSingleUnit) {
            this.isSingleUnit = isSingleUnit;
        }
        boolean isSingleUnit() {
            return isSingleUnit;
        }
        abstract TyParseState next(JavaCaseFormat n);
    }
    static Optional<Integer> typePrefixLength(List<String> nameParts) {
        TyParseState state = TyParseState.START;
        Optional<Integer> typeLength = Optional.empty();
        for (int i = 0; i < nameParts.size(); i++) {
            state = state.next(JavaCaseFormat.from(nameParts.get(i)));
            if (state == TyParseState.REJECT) {
                break;
            }
            if (state.isSingleUnit()) {
                typeLength = Optional.of(i);
            }
        }
        return typeLength;
    }
    enum JavaCaseFormat {
        UPPERCASE, LOWERCASE, UPPER_CAMEL, LOWER_CAMEL;
        static JavaCaseFormat from(String name) {
            Verify.verify(!name.isEmpty());
            boolean firstUppercase = false;
            boolean hasUppercase = false;
            boolean hasLowercase = false;
            boolean first = true;
            for (int i = 0; i < name.length(); i++) {
                char c = name.charAt(i);
                if (!Character.isAlphabetic(c)) {
                    continue;
                }
                if (first) {
                    firstUppercase = Character.isUpperCase(c);
                    first = false;
                }
                hasUppercase |= Character.isUpperCase(c);
                hasLowercase |= Character.isLowerCase(c);
            }
            return firstUppercase ? hasLowercase || name.length() == 1 ? UPPER_CAMEL : UPPERCASE : hasUppercase ? LOWER_CAMEL : LOWERCASE;
        }
    }
}
