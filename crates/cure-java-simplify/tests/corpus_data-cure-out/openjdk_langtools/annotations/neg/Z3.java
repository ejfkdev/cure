enum Color {
    red, green, blue
}

class Colored {
    Color value() 
        default Color.red;
}
