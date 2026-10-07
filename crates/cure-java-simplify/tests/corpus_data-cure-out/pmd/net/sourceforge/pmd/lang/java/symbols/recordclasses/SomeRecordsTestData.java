package net.sourceforge.pmd.lang.java.symbols.recordclasses;

import java.lang.annotation.ElementType;
import java.lang.annotation.Retention;
import java.lang.annotation.RetentionPolicy;
import java.lang.annotation.Target;

@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.TYPE_USE) @interface TypeAnnotation {
}

@Retention(RetentionPolicy.RUNTIME)
@Target(ElementType.RECORD_COMPONENT) @interface RecordAnnot {
}

record Point(int x, int y) {
}

record Varargs(float... varargs) {
}

record Annotated(@Deprecated int x,
                 java.util.List<@TypeAnnotation String> strings) {
}

record AnnotatedForRecord(@RecordAnnot int x) {
}

record GenericBox<T extends java.io.Serializable, X>(T serializable, X x) {
}
