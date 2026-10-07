package spoon.test.template.testclasses;

import spoon.reflect.code.CtExpression;
import spoon.template.BlockTemplate;
import spoon.template.Local;

public class SubstitutionByExpressionTemplate extends BlockTemplate {
    @Override
	public void block() throws Throwable {
        System.out.println(_expression_.S().substring(1));
    }
    CtExpression<String> _expression_;
    @Local
	public SubstitutionByExpressionTemplate(CtExpression<String> expr) {
        this._expression_ = expr;
    }
}
