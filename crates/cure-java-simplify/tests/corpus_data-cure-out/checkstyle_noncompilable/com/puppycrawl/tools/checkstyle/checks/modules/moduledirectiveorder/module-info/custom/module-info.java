module com.example.app {
    requires java.base;

    exports com.example.api;

    // violation below ''uses' directive should be before 'exports' directive.'
    uses com.example.api.Service;
}
