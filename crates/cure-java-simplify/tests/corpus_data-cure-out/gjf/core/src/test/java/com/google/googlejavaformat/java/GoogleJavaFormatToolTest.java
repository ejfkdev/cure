package com.google.googlejavaformat.java;

import static com.google.common.truth.Truth.assertThat;
import static java.nio.charset.StandardCharsets.UTF_8;
import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.InputStream;
import java.util.ServiceLoader;
import javax.tools.Tool;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

@RunWith(JUnit4.class)
public class GoogleJavaFormatToolTest {
    @Test
  public void testUsageOutputAfterLoadingViaToolName() {
        String name = "google-java-format";
        assertThat(ServiceLoader.load(Tool.class).stream().map(ServiceLoader.Provider::get).map(Tool::name)).contains(name);
        Tool format = ServiceLoader.load(Tool.class).stream().filter((provider) -> name.equals(provider.get().name())).findFirst().get().get();
        InputStream in = new ByteArrayInputStream(new byte[0]);
        ByteArrayOutputStream out = new ByteArrayOutputStream();
        ByteArrayOutputStream err = new ByteArrayOutputStream();
        assertThat(format.run(in, out, err, "--help")).isNotEqualTo(0);
        String usage = new String(err.toByteArray(), UTF_8);
        assertThat(usage).containsMatch("http.*/google-java-format");
        assertThat(usage).contains("Usage: google-java-format");
    }
}
