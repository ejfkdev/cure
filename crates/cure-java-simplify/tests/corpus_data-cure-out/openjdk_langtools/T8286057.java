class EnumsCantBeGeneric {
    public enum E1<> {
    }
    public enum E2<T> {
    }
    public enum E3<T, T> {
    }
}
