module com.example.app {
    exports com.example.api;

    requires java.base; // violation ''requires' directive should be before 'exports' directive.'

    opens com.example.model;

    uses com.example.api.Service;

    provides com.example.api.Service with com.example.impl.ServiceImpl;
}
