package spoon.test.prettyprinter.testclasses;

import spoon.reflect.cu.SourcePositionHolder;
import spoon.reflect.declaration.CtElement;
import spoon.reflect.visitor.EarlyTerminatingScanner;

public class ElementScan {
    public void isElementExists(SourcePositionHolder element) {
        EarlyTerminatingScanner<Boolean> scanner = new EarlyTerminatingScanner<Boolean>() {
			@Override
			protected void enter(CtElement e) {
				if (element == e) {
				}
			}
		};
    }
}
