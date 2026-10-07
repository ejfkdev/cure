package spoon.test.template.testclasses.logger;

import spoon.processing.AbstractProcessor;
import spoon.reflect.code.CtBlock;
import spoon.reflect.declaration.CtMethod;
import spoon.template.Template;

public class LoggerTemplateProcessor<T> extends AbstractProcessor<CtMethod<T>> {
    @Override
	public boolean isToBeProcessed(CtMethod<T> candidate) {
        return candidate.getBody() != null && !isSubOfTemplate(candidate);
    }
    private boolean isSubOfTemplate(CtMethod<T> candidate) {
        return candidate.getDeclaringType().isSubtypeOf(getFactory().Type().createReference(Template.class));
    }
    @Override
	public void process(CtMethod<T> element) {
        CtBlock log = new LoggerTemplate(element.getDeclaringType().getSimpleName(), element.getSimpleName(), element.getBody()).apply(element.getDeclaringType());
        element.setBody(log);
    }
}
