class BadStatementInLambda {
    interface SAM {
        Object m();
    }
    SAM t1 = () -> {
    null;
};
    SAM t2 = () -> {
    1;
};
    SAM t3 = () -> {
    6;
};
}
