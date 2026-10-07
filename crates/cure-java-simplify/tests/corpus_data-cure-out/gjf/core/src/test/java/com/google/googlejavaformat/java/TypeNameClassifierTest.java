package com.google.googlejavaformat.java;

import static com.google.common.truth.Truth.assertThat;
import com.google.common.base.Splitter;
import com.google.googlejavaformat.java.TypeNameClassifier.JavaCaseFormat;
import java.util.Optional;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

@RunWith(JUnit4.class)
public final class TypeNameClassifierTest {
    @Test
  public void caseFormat() throws Exception {
        assertThat(JavaCaseFormat.from("CONST")).isEqualTo(JavaCaseFormat.UPPERCASE);
        assertThat(JavaCaseFormat.from("TypeName")).isEqualTo(JavaCaseFormat.UPPER_CAMEL);
        assertThat(JavaCaseFormat.from("fieldName")).isEqualTo(JavaCaseFormat.LOWER_CAMEL);
        assertThat(JavaCaseFormat.from("com")).isEqualTo(JavaCaseFormat.LOWERCASE);
        assertThat(JavaCaseFormat.from("CONST_$")).isEqualTo(JavaCaseFormat.UPPERCASE);
        assertThat(JavaCaseFormat.from("TypeName_$")).isEqualTo(JavaCaseFormat.UPPER_CAMEL);
        assertThat(JavaCaseFormat.from("fieldName_$")).isEqualTo(JavaCaseFormat.LOWER_CAMEL);
        assertThat(JavaCaseFormat.from("com_$")).isEqualTo(JavaCaseFormat.LOWERCASE);
        assertThat(JavaCaseFormat.from("A_$")).isEqualTo(JavaCaseFormat.UPPERCASE);
        assertThat(JavaCaseFormat.from("a_$")).isEqualTo(JavaCaseFormat.LOWERCASE);
        assertThat(JavaCaseFormat.from("_")).isEqualTo(JavaCaseFormat.LOWERCASE);
        assertThat(JavaCaseFormat.from("_A")).isEqualTo(JavaCaseFormat.UPPERCASE);
        assertThat(JavaCaseFormat.from("A")).isEqualTo(JavaCaseFormat.UPPER_CAMEL);
    }
    private static Optional<Integer> getPrefix(String qualifiedName) {
        return TypeNameClassifier.typePrefixLength(Splitter.on('.').splitToList(qualifiedName));
    }
    @Test
  public void typePrefixLength() {
        assertThat(TypeNameClassifier.typePrefixLength(Splitter.on('.').splitToList("fieldName"))).isEmpty();
        assertThat(TypeNameClassifier.typePrefixLength(Splitter.on('.').splitToList("CONST"))).isEmpty();
        assertThat(TypeNameClassifier.typePrefixLength(Splitter.on('.').splitToList("ClassName"))).hasValue(0);
        assertThat(TypeNameClassifier.typePrefixLength(Splitter.on('.').splitToList("com.ClassName"))).hasValue(1);
        assertThat(TypeNameClassifier.typePrefixLength(Splitter.on('.').splitToList("ClassName.foo"))).hasValue(1);
        assertThat(TypeNameClassifier.typePrefixLength(Splitter.on('.').splitToList("com.ClassName.foo"))).hasValue(2);
        assertThat(TypeNameClassifier.typePrefixLength(Splitter.on('.').splitToList("ClassName.foo.bar"))).hasValue(1);
        assertThat(TypeNameClassifier.typePrefixLength(Splitter.on('.').splitToList("com.ClassName.foo.bar"))).hasValue(2);
        assertThat(TypeNameClassifier.typePrefixLength(Splitter.on('.').splitToList("ClassName.CONST"))).hasValue(1);
        assertThat(TypeNameClassifier.typePrefixLength(Splitter.on('.').splitToList("ClassName.varName"))).hasValue(1);
        assertThat(TypeNameClassifier.typePrefixLength(Splitter.on('.').splitToList("ClassName.Inner.varName"))).hasValue(2);
        assertThat(TypeNameClassifier.typePrefixLength(Splitter.on('.').splitToList("com.R.foo"))).hasValue(2);
    }
    @Test
  public void ambiguousClass() {
        assertThat(TypeNameClassifier.typePrefixLength(Splitter.on('.').splitToList("com.google.security.acl.proto2api.ACL.Entry.newBuilder"))).hasValue(7);
        assertThat(TypeNameClassifier.typePrefixLength(Splitter.on('.').splitToList("com.google.security.acl.proto2api.ACL.newBuilder"))).isEmpty();
    }
}
