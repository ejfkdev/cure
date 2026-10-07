package com.puppycrawl.tools.checkstyle.checks.javadoc.missingjavadoctype;

@ThisIsOk2 class InputMissingJavadocTypeSkipAnnotations2 {
}

@com.puppycrawl.tools.checkstyle.checks.javadoc.missingjavadoctype.ThisIsOk2 class InputJavadocTypeSkipAnnotationsFullyQualifiedName2 {
}

@Generated2(value = "some code generator") class InputJavadocTypeAllowedAnnotationByDefault2 {
}

@interface ThisIsOk2 {
}

@interface Generated2 {
    String[] value();
}
