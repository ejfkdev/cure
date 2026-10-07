module com.example.app {
    requires java.base;

    requires java.sql;
    exports com.example.api;


    opens com.example.model;
    // violation below 'All 'requires' directives should be in a single block.'
    requires com.example.extra;
    uses com.example.api.Service;
}
