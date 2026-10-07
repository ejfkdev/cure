String name = "default";
void process(String name) { // violation ''name' hides a field'
    System.out.println(name);
}

void main() { process("test"); }
