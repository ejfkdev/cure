module com.example.app {
    requires java.base;

    requires java.sql; // violation 'Empty line not allowed inside 'requires' directive block.'

    exports com.example.api;


    // violation below 'Empty line not allowed inside 'exports' directive block.'
    exports com.example.internal;

    uses com.example.api.Service;
}
