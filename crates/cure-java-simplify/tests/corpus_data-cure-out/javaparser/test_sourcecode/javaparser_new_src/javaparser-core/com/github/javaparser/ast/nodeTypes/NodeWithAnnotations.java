package com.github.javaparser.ast.nodeTypes;

import java.lang.annotation.Annotation;
import java.util.List;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.expr.MarkerAnnotationExpr;
import com.github.javaparser.ast.expr.NameExpr;
import com.github.javaparser.ast.expr.NormalAnnotationExpr;
import com.github.javaparser.ast.expr.SingleMemberAnnotationExpr;
import static com.github.javaparser.ast.expr.NameExpr.*;

public interface NodeWithAnnotations<T> {
    List<AnnotationExpr> getAnnotations();
    T setAnnotations(List<AnnotationExpr> annotations);
    public default NormalAnnotationExpr addAnnotation(String name) {
        NormalAnnotationExpr normalAnnotationExpr = new NormalAnnotationExpr(name(name), null);
        getAnnotations().add(normalAnnotationExpr);
        normalAnnotationExpr.setParentNode((Node) this);
        return normalAnnotationExpr;
    }
    public default NormalAnnotationExpr addAnnotation(Class<? extends Annotation> clazz) {
        ((Node) this).tryAddImportToParentCompilationUnit(clazz);
        return addAnnotation(clazz.getSimpleName());
    }
    @SuppressWarnings("unchecked")
    public default T addMarkerAnnotation(String name) {
        MarkerAnnotationExpr markerAnnotationExpr = new MarkerAnnotationExpr(name(name));
        getAnnotations().add(markerAnnotationExpr);
        markerAnnotationExpr.setParentNode((Node) this);
        return (T) this;
    }
    public default T addMarkerAnnotation(Class<? extends Annotation> clazz) {
        ((Node) this).tryAddImportToParentCompilationUnit(clazz);
        return addMarkerAnnotation(clazz.getSimpleName());
    }
    @SuppressWarnings("unchecked")
    public default T addSingleMemberAnnotation(String name, String value) {
        SingleMemberAnnotationExpr singleMemberAnnotationExpr = new SingleMemberAnnotationExpr(name(name), name(value));
        getAnnotations().add(singleMemberAnnotationExpr);
        singleMemberAnnotationExpr.setParentNode((Node) this);
        return (T) this;
    }
    public default T addSingleMemberAnnotation(Class<? extends Annotation> clazz, String value) {
        ((Node) this).tryAddImportToParentCompilationUnit(clazz);
        return addSingleMemberAnnotation(clazz.getSimpleName(), value);
    }
    public default boolean isAnnotationPresent(String annotationName) {
        return getAnnotations().stream().anyMatch((a) -> a.getName().getName().equals(annotationName));
    }
    public default boolean isAnnotationPresent(Class<? extends Annotation> annotationClass) {
        return isAnnotationPresent(annotationClass.getSimpleName());
    }
    public default AnnotationExpr getAnnotationByName(String annotationName) {
        return getAnnotations().stream().filter((a) -> a.getName().getName().equals(annotationName)).findFirst().orElse(null);
    }
    public default AnnotationExpr getAnnotationByClass(Class<? extends Annotation> annotationClass) {
        return getAnnotationByName(annotationClass.getSimpleName());
    }
}
