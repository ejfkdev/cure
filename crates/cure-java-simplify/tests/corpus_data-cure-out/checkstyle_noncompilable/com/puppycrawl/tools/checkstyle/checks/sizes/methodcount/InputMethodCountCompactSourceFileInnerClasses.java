void run1() { // violation 'Total number of methods is 2 (max allowed is 1).'
    Runnable r = new Runnable() {
        public void run() {}
    };
    class Local { // violation 'Total number of methods is 2 (max allowed is 1).'
        void h1() {}

        void h2() {}
    }
}

void main() {}
