module com.example.app {
    requires java.base;
    // comment between blocks is not an empty line
    exports com.example.api;
    // violation above 'should be separated from the previous block by exactly one empty line'
}
