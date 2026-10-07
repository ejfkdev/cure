package spoon.test.template.testclasses;

import spoon.reflect.declaration.CtClass;
import spoon.reflect.declaration.CtType;
import spoon.template.Local;
import spoon.template.Parameter;
import spoon.template.StatementTemplate;
import spoon.template.Substitution;
import spoon.template.Template;
import java.sql.Statement;

public class SimpleTemplate extends StatementTemplate {
    @Parameter String _parameter_;
    @Local
    public SimpleTemplate(String parameter) {
        _parameter_ = parameter;
    }
    @Override
    public CtClass apply(CtType targetType) {
        Substitution.insertAll(targetType, this);
        return (CtClass) targetType;
    }
    @Override
    public void statement() throws Throwable {
        System.out.println(_parameter_);
    }
}
