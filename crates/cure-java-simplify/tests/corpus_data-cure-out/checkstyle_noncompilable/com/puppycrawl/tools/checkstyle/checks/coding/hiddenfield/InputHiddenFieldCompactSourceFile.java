String name = "default";
int count = 0;
void process(String name, int count) {
    // 2 violations above:
    //                   ''name' hides a field.'
    //                   ''count' hides a field.'
    System.out.println(name + ": " + count);
}

void main() { process("test", 42); }
