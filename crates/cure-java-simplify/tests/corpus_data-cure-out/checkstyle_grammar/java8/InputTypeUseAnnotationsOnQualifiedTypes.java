package com.puppycrawl.tools.checkstyle.grammar.java8;

import java.awt.geom.Rectangle2D;
import java.lang.annotation.ElementType;
import java.lang.annotation.Target;

public class InputTypeUseAnnotationsOnQualifiedTypes {
    Rectangle2D.Double rect = null;
    public final Rectangle2D.Double getRect1() {
        return new Rectangle2D.Double();
    }
    public final Rectangle2D.Double getRect2() {
        return new Rectangle2D.Double();
    }
    public final Rectangle2D.Double getRect3() {
        return null;
    }
}

@Target({ ElementType.TYPE_USE }) @interface Ann {
}
