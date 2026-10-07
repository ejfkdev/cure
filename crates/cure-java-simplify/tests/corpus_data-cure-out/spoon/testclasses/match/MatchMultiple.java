package spoon.test.template.testclasses.match;

import spoon.pattern.Pattern;
import spoon.pattern.PatternBuilder;
import spoon.pattern.PatternBuilderHelper;
import spoon.pattern.Quantifier;
import spoon.reflect.code.CtLiteral;
import spoon.reflect.declaration.CtType;
import spoon.reflect.meta.ContainerKind;
import spoon.testing.utils.ModelUtils;
import static java.lang.System.out;

public class MatchMultiple {
    public static Pattern createPattern(Quantifier matchingStrategy, Integer minCount, Integer maxCount) throws Exception {
        CtType<?> type = ModelUtils.buildClass(MatchMultiple.class);
        return PatternBuilder.create(new PatternBuilderHelper(type).setBodyOfMethod("matcher1").getPatternElements()).configurePatternParameters((pb) -> {
            pb.parameter("statements").byReferenceName("statements").setContainerKind(ContainerKind.LIST);
            if (matchingStrategy != null) {
                pb.setMatchingStrategy(matchingStrategy);
            }
            if (minCount != null) {
                pb.setMinOccurrence(minCount);
            }
            if (maxCount != null) {
                pb.setMaxOccurrence(maxCount);
            }
            pb.parameter("printedValue").byFilter((CtLiteral<?> literal) -> "something".equals(literal.getValue()));
        }).build();
    }
    public void matcher1() {
        statements();
        System.out.println("something");
    }
    void statements() {}
    public void testMatch1() {
        int i = 0;
        i++;
        System.out.println(i);
        out.println("Xxxx");
        System.out.println((String) null);
        System.out.println("last one");
    }
}
