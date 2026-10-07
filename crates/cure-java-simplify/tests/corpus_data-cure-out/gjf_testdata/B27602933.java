class B27602933 {
    {
        try {} catch (A | B x) {}
        try {} catch (@SuppressWarnings("unused") IllegalArgumentException | RuntimeException e) {}
    }
}
