package spoon.test.position.testclasses;

import java.util.List;

public class FooForEach {
    public void m(List<String> items) {
        for (String item : items) {}
        for (String item : items) {}
        for (String i : items) 
            this.getClass();
    }
}
