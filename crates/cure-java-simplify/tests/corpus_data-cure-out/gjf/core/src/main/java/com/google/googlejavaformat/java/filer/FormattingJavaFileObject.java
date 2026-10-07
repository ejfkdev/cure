package com.google.googlejavaformat.java.filer;

import static com.google.common.base.Preconditions.checkNotNull;
import com.google.common.io.CharSink;
import com.google.common.io.CharSource;
import com.google.googlejavaformat.java.Formatter;
import com.google.googlejavaformat.java.FormatterException;
import java.io.IOException;
import java.io.Writer;
import javax.annotation.processing.Messager;
import javax.tools.Diagnostic;
import javax.tools.ForwardingJavaFileObject;
import javax.tools.JavaFileObject;
import org.jspecify.annotations.Nullable;

final class FormattingJavaFileObject extends ForwardingJavaFileObject<JavaFileObject> {
    private static final int DEFAULT_FILE_SIZE = 80 * 500;
    private final Formatter formatter;
    private final Messager messager;
    FormattingJavaFileObject(JavaFileObject delegate, Formatter formatter, @Nullable Messager messager) {
        super(checkNotNull(delegate));
        this.formatter = checkNotNull(formatter);
        this.messager = messager;
    }
    @Override
  public Writer openWriter() throws IOException {
        StringBuilder stringBuilder = new StringBuilder(DEFAULT_FILE_SIZE);
        return new Writer() {
      @Override
      public void write(char[] chars, int start, int end) throws IOException {
        stringBuilder.append(chars, start, end - start);
      }

      @Override
      public void write(String string) throws IOException {
        stringBuilder.append(string);
      }

      @Override
      public void flush() throws IOException {}

      @Override
      public void close() throws IOException {
        try {
          formatter.formatSource(
              CharSource.wrap(stringBuilder),
              new CharSink() {
                @Override
                public Writer openStream() throws IOException {
                  return fileObject.openWriter();
                }
              });
        } catch (FormatterException e) {
          // An exception will happen when the code being formatted has an error. It's better to
          // log the exception and emit unformatted code so the developer can view the code which
          // caused a problem.
          try (Writer writer = fileObject.openWriter()) {
            writer.append(stringBuilder.toString());
          }
          if (messager != null) {
            messager.printMessage(Diagnostic.Kind.NOTE, "Error formatting " + getName());
          }
        }
      }
    };
    }
}
