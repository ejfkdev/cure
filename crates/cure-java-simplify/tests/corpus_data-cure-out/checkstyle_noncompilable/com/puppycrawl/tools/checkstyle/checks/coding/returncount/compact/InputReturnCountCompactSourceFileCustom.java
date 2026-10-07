void main() {
}

int classify(boolean active) { // violation 'Return count is 2'
    if (active) {
        return 1;
    }
    return 0;
}

void execute() { // violation 'Return count is 1'
    return;
}

boolean equals(int value) { // violation 'Return count is 2'
    if (value == 0) {
        return true;
    }
    return false;
}

int ignored(int value) {
    if (value < 0) {
        return -1;
    }
    if (value == 0) {
        return 0;
    }
    return 1;
}

int atLimit() {
    return 1;
}
