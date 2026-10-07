package com.google.googlejavaformat.java.filer;

import static com.google.common.base.Preconditions.checkNotNull;
import com.google.googlejavaformat.java.Formatter;
import java.io.IOException;
import javax.annotation.processing.Filer;
import javax.annotation.processing.Messager;
import javax.annotation.processing.ProcessingEnvironment;
import javax.lang.model.element.Element;
import javax.tools.FileObject;
import javax.tools.JavaFileManager;
import javax.tools.JavaFileObject;
import org.jspecify.annotations.Nullable;

public final class FormattingFiler implements Filer {
    private final Filer delegate;
    private final Formatter formatter = new Formatter();
    private final Messager messager;
    public static Filer create(ProcessingEnvironment processingEnv) {
        Filer delegate = processingEnv.getFiler();
        return processingEnv.getOptions().containsKey("experimental_turbine_hjar") ? delegate : new FormattingFiler(delegate, processingEnv.getMessager());
    }
    @Deprecated
  public FormattingFiler(Filer delegate) {
        this(delegate, null);
    }
    @Deprecated
  public FormattingFiler(Filer delegate, @Nullable Messager messager) {
        this.delegate = checkNotNull(delegate);
        this.messager = messager;
    }
    @Override
  public JavaFileObject createSourceFile(CharSequence name, Element... originatingElements) throws IOException {
        return new FormattingJavaFileObject(delegate.createSourceFile(name, originatingElements), formatter, messager);
    }
    @Override
  public JavaFileObject createClassFile(CharSequence name, Element... originatingElements) throws IOException {
        return delegate.createClassFile(name, originatingElements);
    }
    @Override
  public FileObject createResource(JavaFileManager.Location location, CharSequence pkg, CharSequence relativeName, Element... originatingElements) throws IOException {
        return delegate.createResource(location, pkg, relativeName, originatingElements);
    }
    @Override
  public FileObject getResource(JavaFileManager.Location location, CharSequence pkg, CharSequence relativeName) throws IOException {
        return delegate.getResource(location, pkg, relativeName);
    }
}
