package spoon.test.template.testclasses;

import spoon.reflect.code.CtLiteral;
import spoon.template.TemplateParameter;

public class SecurityCheckerTemplate {
    public TemplateParameter<ContextHelper> _ctx_;
    public CtLiteral<String> _p_;
    public void matcher1() {
        if (!_ctx_.S().hasPermission(_p_.S())) {
            throw new SecurityException();
        }
    }
}
