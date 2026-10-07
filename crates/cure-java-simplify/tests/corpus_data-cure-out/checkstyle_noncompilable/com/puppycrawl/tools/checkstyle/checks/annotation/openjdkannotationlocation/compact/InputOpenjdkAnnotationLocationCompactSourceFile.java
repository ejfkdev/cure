int field1 = 0;
String field2 = "ok";
int field3 = 0;
String field4 = "ok";
int badField6 = 0;
// violation above 'Annotations must be on a separate line from 'badField6'.'
int goodField6;
int good;

// violation below 'Annotations must be on a separate line from 'badField7'.'
int
        badField7;
void main() {
    @SuppressWarnings("unused")
    @Deprecated int local1 = 0;
    // violation above 'Annotations must be on a separate line from 'local1'.'
    @Deprecated int local2 = 0;
    System.out.println(local1 + local2 + field1);
}

@Deprecated void helperMethod() {}

@Helper @Deprecated
void helperMethodOne() {}

@Helper
@Deprecated
void helperMethodTwo() {}

@Helper @Deprecated
@Special
void helperMethodThree() {}
// violation above """Annotations on 'helperMethodThree' must be all on one line or
// all on separate lines."""

@Helper @Deprecated @Special void helperMethodFour() {}

@interface Helper {}
@interface Special {}

class Nested {
    @SuppressWarnings("unused") int nestedField1 = 0;

    @SuppressWarnings("unused")
    int nestedField3 = 0;

    void nestedMethod() {
        @SuppressWarnings("unused") int nestedLocal = 0;
        System.out.println(nestedLocal);
    }
}
