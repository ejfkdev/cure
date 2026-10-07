package com.google.googlejavaformat.java;

import static com.google.common.collect.Sets.toImmutableEnumSet;
import com.google.auto.service.AutoService;
import java.io.InputStream;
import java.io.OutputStream;
import java.io.PrintStream;
import java.util.Arrays;
import java.util.Set;
import javax.lang.model.SourceVersion;
import javax.tools.Tool;

@AutoService(Tool.class)
public class GoogleJavaFormatTool implements Tool {
    @Override
  public String name() {
        return "google-java-format";
    }
    @Override
  public Set<SourceVersion> getSourceVersions() {
        return Arrays.stream(SourceVersion.values()).collect(toImmutableEnumSet());
    }
    @Override
  public int run(InputStream in, OutputStream out, OutputStream err, String... args) {
        PrintStream outStream = new PrintStream(out);
        PrintStream errStream = new PrintStream(err);
        try {
            return Main.main(in, outStream, errStream, args);
        } catch (RuntimeException e) {
            errStream.print(e.getMessage());
            errStream.flush();
            return 1;
        }
    }
}
