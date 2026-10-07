package com.puppycrawl.tools.checkstyle.grammar;

public class InputEmbeddedNullChar {
    public void doSomething() {
        String cctCxlMsg = ":\u001EET:\u001EOE:\u001E}}\u0000";
    }
}
