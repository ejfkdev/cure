void main() {
    int value;
    String result = Integer.toString(value = 2); // violation 'Inner assignments should be avoided'
}
