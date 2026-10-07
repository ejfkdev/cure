int field;
void method() {

}; // violation 'Unnecessary semicolon.'
void main() {
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
}

class LeadingSemicolon {
}

enum EnumLeadingSemicolon {
}

class CleanClass {
}
