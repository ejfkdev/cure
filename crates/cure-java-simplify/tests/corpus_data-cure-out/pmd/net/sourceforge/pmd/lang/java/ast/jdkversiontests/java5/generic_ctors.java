class GenericConstructor {
    Object ok;
    ();
    Object okWithPackage;
    ();
    Object ok2;
    ();
    Object o3;
    ().new <String>NonStatic();
    Object o4;
    ();
    Object o5;
    ().new <String>GenericInner<String>();
}

class Outer {
    static class Inner {
    }
    class NonStatic {
    }
}

class GenericOuter<T> {
    class GenericInner<U> {
    }
}
