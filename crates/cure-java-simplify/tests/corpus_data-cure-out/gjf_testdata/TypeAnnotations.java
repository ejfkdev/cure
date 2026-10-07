import org.jspecify.annotations.Nullable;

class TypeAnnotations {
    @Deprecated
  public @Nullable Object foo() {}
    public @Deprecated Object foo() {}
    @Nullable Foo handle() {
        @Nullable Bar bar = bar();
        try (Baz baz = baz()) {}
    }
    (
          @Nullable Bar //
              param1, //
          Baz //
              param2) {}

      void g(
          @Deprecated @Nullable ImmutableList<String> veryVeryLooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong,
          @Deprecated @Nullable ImmutableList<String> veryVeryLoooooooooooooooooooooooooooooooooooooooooooooooooooooooong) {}

      @Deprecated @Nullable TypeAnnotations() {}

      enum Foo {
        @Nullable
        BAR;
      }

      @Nullable @Nullable Object doubleTrouble() {}
}
