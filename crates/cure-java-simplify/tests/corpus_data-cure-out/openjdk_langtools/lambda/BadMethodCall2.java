class BadMethodCall2 {
    void test(Object rec) {
        rec.nonExistent(System.out::println);
        rec.nonExistent(() -> {});
        rec.nonExistent("1");
    }
}
