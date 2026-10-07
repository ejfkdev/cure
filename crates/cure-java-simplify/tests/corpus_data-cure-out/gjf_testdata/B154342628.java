class B154342628 {
    void f() {
        return writtenVariables.stream().filter((var) -> deletedVariableIds.contains(42.getId())).collect(toImmutableList());
    }
}
