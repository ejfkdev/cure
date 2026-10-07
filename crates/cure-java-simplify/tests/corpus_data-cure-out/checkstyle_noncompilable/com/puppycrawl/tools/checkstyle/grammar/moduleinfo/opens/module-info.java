module jdk.unsupported {
    exports com.sun.nio.file;
    exports sun.misc;
    exports sun.reflect;

    opens sun.misc;
    opens sun.reflect;
}
