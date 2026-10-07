package spoon.test.generics.testclasses;

import java.util.ArrayList;
import java.util.List;

public class ExtendedPaella<T extends List<T>> extends Paella {
    class InnerPaella<L extends T> {
        public <L extends ArrayList> T innerMachin(L param) {
            return null;
        }
        public <T extends String> L innerToto(T param) {
            return null;
        }
    }
    public <T extends ArrayList> T machin() {
        return null;
    }
    public T toto(T param) {
        return null;
    }
}
