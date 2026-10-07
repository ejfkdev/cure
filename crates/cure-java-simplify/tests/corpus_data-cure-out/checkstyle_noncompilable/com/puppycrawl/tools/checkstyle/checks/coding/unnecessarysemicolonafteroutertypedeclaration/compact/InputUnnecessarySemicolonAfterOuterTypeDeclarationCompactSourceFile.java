void main() {
    class LocalClass {

    };
    System.out.println("Hello!");
}
class MemberClass {
}

interface MemberInterface {
}

enum MemberEnum {
}

@interface MemberAnnotation {
}

record MemberRecord() {
}

class OuterWithNested {
    class Nested {
    }
    enum NestedEnum {
    }
}

class CleanClass {
}

interface CleanInterface {
}

enum CleanEnum {
}

@interface CleanAnnotation {
}

record CleanRecord() {
}
