package spoon.test.template.testclasses.inheritance;

import spoon.reflect.code.CtExpression;
import spoon.reflect.code.CtInvocation;
import spoon.reflect.declaration.CtParameter;
import spoon.template.Local;
import spoon.template.Parameter;
import java.util.ArrayList;
import java.util.LinkedList;
import java.util.List;

public class SubTemplate extends SuperTemplate {
    public void toBeOverriden() {
        super.toBeOverriden();
    }
    public void methodWithTemplatedParameters(Object params) {
        ArrayList l;
        List o = (ArrayList) new ArrayList();
        invocation.S();
        {
            for (Object x : intValues) {
                System.out.println(x);
            }
        }
        for (Object x : intValues) {
            System.out.println(x);
        }
        for (Object x : intValues) {
            System.out.println(x);
        }
        for (Object x : intValues) 
            System.out.println(x);
        for (Object x : o) {
            System.out.println(x);
        }
        l = (ArrayList) o;
    }
    List var = null;
    public void methodWithFieldAccess() {
        List o = (ArrayList) new ArrayList();
        ArrayList l;
        var = o;
        l = (ArrayList) var;
    }
    void var() {}
    @Parameter
	public List<CtParameter> params;
    @Parameter("var")
	public String param_var = "newVarName";
    @Parameter Class ArrayList = LinkedList.class;
    @Parameter
	public CtInvocation invocation;
    @Parameter
	public CtExpression[] intValues;
    @Local
	public void ignoredMethod() {}
    class InnerClass {
    }
}
