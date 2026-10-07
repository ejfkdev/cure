int field1 = 0;
String field2 = "ok";
int field3 = 0;
String field4 = "ok";

// 2 violations 3 lines below:
//    "Annotation 'SuppressWarnings' should be alone on line."
//    "Annotation 'Deprecated' should be alone on line."
int field5 = 0;
int field6 = 0;
void main() {
    @SuppressWarnings("unused") int local1 = 0;
    @Deprecated int local2 = 0;
    System.out.println(local1 + local2 + field1);
}

class Nested {
    // violation below 'Annotation 'SuppressWarnings' should be alone on line.'
    @SuppressWarnings("unused") int nestedField1 = 0;

    @Deprecated String nestedField2 = "ok";

    @SuppressWarnings("unused")
    int nestedField3 = 0;

    void nestedMethod() {
        @SuppressWarnings("unused") int nestedLocal = 0;
        System.out.println(nestedLocal);
    }
}
