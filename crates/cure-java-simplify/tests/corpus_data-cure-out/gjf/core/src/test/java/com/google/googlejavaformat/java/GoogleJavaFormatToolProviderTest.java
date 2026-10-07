package com.google.googlejavaformat.java;

import static com.google.common.truth.Truth.assertThat;
import java.io.PrintWriter;
import java.io.StringWriter;
import java.util.ServiceLoader;
import java.util.spi.ToolProvider;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

@RunWith(JUnit4.class)
public class GoogleJavaFormatToolProviderTest {
    @Test
  public void testUsageOutputAfterLoadingViaToolName() {
        String name = "google-java-format";
        assertThat(ServiceLoader.load(ToolProvider.class).stream().map(ServiceLoader.Provider::get).map(ToolProvider::name)).contains(name);
        ToolProvider format = ToolProvider.findFirst(name).get();
        StringWriter out = new StringWriter();
        StringWriter err = new StringWriter();
        assertThat(format.run(new PrintWriter(out, true), new PrintWriter(err, true), "--help")).isNotEqualTo(0);
        String usage = err.toString();
        assertThat(usage).containsMatch("http.*/google-java-format");
        assertThat(usage).contains("Usage: google-java-format");
    }
}
