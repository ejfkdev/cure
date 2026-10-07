package com.google.googlejavaformat.java;

import com.google.auto.service.AutoService;
import java.io.PrintWriter;
import java.util.spi.ToolProvider;

@AutoService(ToolProvider.class)
public class GoogleJavaFormatToolProvider implements ToolProvider {
    @Override
  public String name() {
        return "google-java-format";
    }
    @Override
  public int run(PrintWriter out, PrintWriter err, String... args) {
        try {
            return Main.main(System.in, out, err, args);
        } catch (RuntimeException e) {
            err.print(e.getMessage());
            err.flush();
            return 1;
        }
    }
}
