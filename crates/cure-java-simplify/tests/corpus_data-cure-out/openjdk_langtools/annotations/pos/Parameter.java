package annotation.parameter;

@interface A {
}

class T {
    void f(final @A int x) {}
    void g(@A final int y, int[] x) {
        for (int a : x) {}
        for (int a : x) 
            break;
        for (int a : x) 
            break;
    }
}
