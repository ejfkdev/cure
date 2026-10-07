void main() {
    int i = 0;
    while (i++ < 3);
    for (int j = 0; j < 3; j++);
    if (i > 0) i--; // violation ''if' construct must use '{}'s'
}
