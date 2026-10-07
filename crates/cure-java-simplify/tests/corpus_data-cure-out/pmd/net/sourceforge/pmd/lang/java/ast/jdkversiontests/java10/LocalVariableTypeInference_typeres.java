package net.sourceforge.pmd.typeresolution.testdata.dummytypes;

import java.util.List;
import java.util.ArrayList;

public class MyList {
    public void checkIterator(List<?> other) {
        other.iterator().hasNext();
    }
}
