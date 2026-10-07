package spoon.test.filters.testclasses;

import java.util.ArrayList;

public class FieldAccessFilterTacos extends ArrayList {
    private int myfield = 0;
    FieldAccessFilterTacos() {
        super();
        this.myfield = 0;
    }
    public void m() {
        myfield = super.size();
        Object o = super.get(myfield);
    }
}
