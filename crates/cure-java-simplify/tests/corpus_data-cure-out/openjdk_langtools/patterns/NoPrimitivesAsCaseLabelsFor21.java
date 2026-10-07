import java.io.IOException;
import java.lang.classfile.Attributes;
import java.lang.classfile.ClassFile;
import java.lang.classfile.ClassModel;
import java.lang.classfile.constantpool.ClassEntry;
import java.lang.classfile.constantpool.LoadableConstantEntry;
import java.lang.classfile.constantpool.MethodHandleEntry;
import java.lang.constant.DirectMethodHandleDesc;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.concurrent.atomic.AtomicBoolean;
import toolbox.JavacTask;
import toolbox.TestRunner;
import toolbox.ToolBox;

public class NoPrimitivesAsCaseLabelsFor21 extends TestRunner {
    ToolBox tb;
    public static void main(String... args) throws Exception {
        new NoPrimitivesAsCaseLabelsFor21().runTests();
    }
    NoPrimitivesAsCaseLabelsFor21() {
        super(System.err);
        tb = new ToolBox();
    }
    public void runTests() throws Exception {
        runTests((m) -> new Object[] {Paths.get(m.getName())});
    }
    @Test
    public void testExhaustiveSealedClasses(Path base) throws Exception {
        Path current = base.resolve(".");
        Path src = current.resolve("src");
        tb.writeJavaFiles(src, """
                          package test;
                          public class Test {
                                private int test(Object obj) {
                                  return switch (obj) {
                                      case R1(String s1, String s2) when s1.isEmpty() -> 0;
                                      case R1(String s1, String s2) -> 1;
                                      case R2(int i1, int i2) when i1 == 0 -> 2;
                                      case R2(int i1, int i2) -> 3;
                                      default -> 4;
                                  };
                              }
                              record R1(String s1, String s2) {}
                              record R2(int i1, int i2) {}
                          }
                          """);
        Path classes = current.resolve("classes");
        Files.createDirectories(classes);
        for (String version : new String[] {"23", System.getProperty("java.specification.version")}) {
            new JavacTask(tb).options("--release", version).outdir(classes).files(tb.findJavaFiles(src)).run().writeAll();
            String primitivesInBoostrapArgsForNewer = findPrimitiveBootstrapArguments(classes.resolve("test").resolve("Test.class"));
            if (!primitivesInBoostrapArgsForNewer.contains("I-Ljava/lang/Class")) {
                throw new AssertionError("Expected primitive types in switch bootstrap arguments: " + primitivesInBoostrapArgsForNewer);
            }
        }
        for (String version : new String[] {"21", "22"}) {
            new JavacTask(tb).options("--release", version).outdir(classes).files(tb.findJavaFiles(src)).run().writeAll();
            String primitivesInBoostrapArgsForOlder = findPrimitiveBootstrapArguments(classes.resolve("test").resolve("Test.class"));
            if (!primitivesInBoostrapArgsForOlder.isEmpty()) {
                throw new AssertionError("Unexpected primitive types in switch bootstrap arguments: " + primitivesInBoostrapArgsForOlder);
            }
        }
    }
    private String findPrimitiveBootstrapArguments(Path forFile) throws IOException {
        AtomicBoolean hasTypeSwitchBootStrap = new AtomicBoolean();
        StringBuilder nonClassInTypeSwitchBootStrap = new StringBuilder();
        ClassFile.of().parse(forFile).findAttribute(Attributes.bootstrapMethods()).orElseThrow().bootstrapMethods().stream().filter((bme) -> isTypeSwitchBoostrap(bme.bootstrapMethod())).forEach((bme) -> {
            hasTypeSwitchBootStrap.set(true);
            for (LoadableConstantEntry e : bme.arguments()) {
                if (!(e instanceof ClassEntry)) {
                    nonClassInTypeSwitchBootStrap.append(String.valueOf(e));
                }
            }
        });
        if (!hasTypeSwitchBootStrap.get()) {
            throw new AssertionError("Didn't find any typeSwitch bootstraps!");
        }
        return nonClassInTypeSwitchBootStrap.toString();
    }
    private static boolean isTypeSwitchBoostrap(MethodHandleEntry entry) {
        DirectMethodHandleDesc desc = entry.asSymbol();
        return "Ljava/lang/runtime/SwitchBootstraps;".equals(desc.owner().descriptorString()) && "typeSwitch".equals(desc.methodName()) && "(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/invoke/MethodType;[Ljava/lang/Object;)Ljava/lang/invoke/CallSite;".equals(desc.lookupDescriptor());
    }
}
