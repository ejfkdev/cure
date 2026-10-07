package spoon.test.template.testclasses;

import spoon.template.ExtensionTemplate;
import spoon.template.Local;

public class ObjectIsNotParamTemplate extends ExtensionTemplate {
    Object o = "XXX";
    void method() {}
    @Local
	public ObjectIsNotParamTemplate() {}
}
