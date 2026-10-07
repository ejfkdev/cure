package com.google.googlejavaformat.java;

import static com.google.common.truth.Truth.assertThat;
import static org.junit.Assert.fail;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

@RunWith(JUnit4.class)
public class CommandLineFlagsTest {
    @Test
  public void formatInPlaceRequiresAtLeastOneFile() throws UsageException {
        try {
            Main.processArgs("-i");
            fail();
        } catch (UsageException e) {}
        try {
            Main.processArgs("-i", "-");
            fail();
        } catch (UsageException e) {}
        Main.processArgs("-i", "Foo.java");
        Main.processArgs("-i", "Foo.java", "Bar.java");
    }
    @Test
  public void formatASubsetRequiresExactlyOneFile() throws UsageException {
        Main.processArgs("-lines", "10", "Foo.java");
        try {
            Main.processArgs("-lines", "10");
            fail();
        } catch (UsageException e) {}
        try {
            Main.processArgs("-lines", "10", "Foo.java", "Bar.java");
            fail();
        } catch (UsageException e) {}
        Main.processArgs("-offset", "10", "-length", "10", "Foo.java");
        try {
            Main.processArgs("-offset", "10", "-length", "10");
            fail();
        } catch (UsageException e) {}
        try {
            Main.processArgs("-offset", "10", "-length", "10", "Foo.java", "Bar.java");
            fail();
        } catch (UsageException e) {}
    }
    @Test
  public void numberOfOffsetsMustMatchNumberOfLengths() throws UsageException {
        Main.processArgs("-offset", "10", "-length", "20", "Foo.java");
        try {
            Main.processArgs("-offset", "10", "-length", "20", "-offset", "50", "Foo.java");
            fail();
        } catch (UsageException e) {}
        try {
            Main.processArgs("-offset", "10", "-length", "20", "-length", "50", "Foo.java");
            fail();
        } catch (UsageException e) {}
    }
    @Test
  public void noFilesToFormatRequiresEitherHelpOrVersion() throws UsageException {
        Main.processArgs("-version");
        Main.processArgs("-help");
        try {
            Main.processArgs();
            fail();
        } catch (UsageException e) {}
        try {
            Main.processArgs("-aosp");
            fail();
        } catch (UsageException e) {}
    }
    @Test
  public void stdinAndFiles() {
        try {
            Main.processArgs("-", "A.java");
            fail();
        } catch (UsageException e) {
            assertThat(e).hasMessageThat().contains("cannot format from standard input and files simultaneously");
        }
    }
    @Test
  public void inPlaceStdin() {
        try {
            Main.processArgs("-i", "-");
            fail();
        } catch (UsageException e) {
            assertThat(e).hasMessageThat().contains("in-place formatting was requested but no files were provided");
        }
    }
    @Test
  public void inPlaceDryRun() {
        try {
            Main.processArgs("-i", "-n", "A.java");
            fail();
        } catch (UsageException e) {
            assertThat(e).hasMessageThat().contains("cannot use --dry-run and --in-place at the same time");
        }
    }
    @Test
  public void assumeFileNameOnlyWorksWithStdin() {
        try {
            Main.processArgs("--assume-filename=Foo.java", "Foo.java");
            fail();
        } catch (UsageException e) {
            assertThat(e).hasMessageThat().contains("--assume-filename is only supported when formatting standard input");
        }
    }
}
