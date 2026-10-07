module com.example.app {
    requires java.base;

    exports com.example.api to

            com.example.other;

    opens com.example.model;
}
