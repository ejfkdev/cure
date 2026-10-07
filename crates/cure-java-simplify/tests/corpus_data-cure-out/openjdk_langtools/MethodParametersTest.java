import java.lang.classfile.*;
import java.lang.classfile.attribute.*;
import com.sun.tools.javac.code.Symtab;
import com.sun.tools.javac.file.JavacFileManager;
import com.sun.tools.javac.main.Main;
import com.sun.tools.javac.util.Context;
import com.sun.tools.javac.util.Name;
import com.sun.tools.javac.util.Names;
import java.io.*;
import javax.lang.model.element.*;
import java.nio.file.Files;
import java.util.*;

public class MethodParametersTest {
    static final String Foo_name = "Foo";
    static final String Foo_contents = "public class Foo {\n  Foo() {}\n  void foo0() {}\n  void foo2(int j, int k) {}\n" + "}";
    static final String Bar_name = "Bar";
    static final String Bar_contents = "public class Bar {\n  Bar(int i) {}  Foo foo() { return new Foo(); }\n" + "}";
    static final String Baz_name = "Baz";
    static final String Baz_contents = "public class Baz {\n  int baz;  Baz(int i) {}" + "}";
    static final String Qux_name = "Qux";
    static final String Qux_contents = "public class Qux extends Baz {\n  Qux(int i) { super(i); }" + "}";
    static final File classesdir = new File("methodparameters");
    public static void main(String... args) throws Exception {
        new MethodParametersTest().run();
    }
    void run() throws Exception {
        classesdir.mkdir();
        File Foo_java = writeFile(classesdir, Foo_name + ".java", Foo_contents);
        File Bar_java = writeFile(classesdir, Bar_name + ".java", Bar_contents);
        File Baz_java = writeFile(classesdir, Baz_name + ".java", Baz_contents);
        System.err.println("Test compile with -parameter");
        compile("-parameters", "-d", classesdir.getPath(), Foo_java.getPath());
        System.err.println("Test compile with classfile containing MethodParameter attributes");
        compile("-parameters", "-d", classesdir.getPath(), "-cp", classesdir.getPath(), Bar_java.getPath());
        System.err.println("Examine class foo");
        checkFoo();
        checkBar();
        System.err.println("Test debug information conflict");
        compile("-g", "-parameters", "-d", classesdir.getPath(), "-cp", classesdir.getPath(), Baz_java.getPath());
        System.err.println("Introducing debug information conflict");
        Baz_java.delete();
        modifyBaz(false);
        System.err.println("Checking language model");
        inspectBaz();
        System.err.println("Permuting attributes");
        modifyBaz(true);
        System.err.println("Checking language model");
        inspectBaz();
        if (0 != errors) 
            throw new Exception("MethodParameters test failed with " + errors + " errors");
    }
    void inspectBaz() throws Exception {
        File Qux_java = writeFile(classesdir, Qux_name + ".java", Qux_contents);
        String[] args = {"-parameters", "-d", classesdir.getPath(), "-cp", classesdir.getPath(), Qux_java.getPath()};
        StringWriter sw = new StringWriter();
        PrintWriter pw = new PrintWriter(sw);
        Context context = new Context();
        Main comp = new Main("javac", pw);
        JavacFileManager.preRegister(context);
        comp.compile(args, context);
        pw.close();
        String out = sw.toString();
        if (out.length() > 0) 
            System.err.println(out);
        com.sun.tools.javac.code.ClassFinder cf = com.sun.tools.javac.code.ClassFinder.instance(context);
        Name name = Names.instance(context).fromString(Baz_name);
        Symtab syms = Symtab.instance(context);
        Element baz = cf.loadClass(syms.unnamedModule, name);
        for (Element e : baz.getEnclosedElements()) {
            if (e instanceof ExecutableElement ee) {
                List<? extends VariableElement> params = ee.getParameters();
                if (1 != params.size()) 
                    throw new Exception("Classfile Baz badly formed: wrong number of methods");
                VariableElement param = params.get(0);
                if (!param.getSimpleName().contentEquals("baz")) {
                    errors++;
                    System.err.println("javac did not correctly resolve the metadata conflict, parameter's name reads as " + param.getSimpleName());
                } else 
                    System.err.println("javac did correctly resolve the metadata conflict");
            }
        }
    }
    void modifyBaz(boolean flip) throws Exception {
        File Baz_class = new File(classesdir, Baz_name + ".class");
        ClassModel baz = ClassFile.of().parse(Baz_class.toPath());
        if (baz.methods().size() != 1) 
            throw new Exception("Classfile Baz badly formed: wrong number of methods");
        if (!baz.methods().get(0).methodName().equalsString("<init>")) 
            throw new Exception("Classfile Baz badly formed: method has name " + baz.methods().get(0).methodName().stringValue());
        MethodParametersAttribute mpattr = baz.methods().get(0).findAttribute(Attributes.methodParameters()).orElse(null);
        CodeAttribute cattr = baz.methods().get(0).findAttribute(Attributes.code()).orElse(null);
        if (null == mpattr) 
            throw new Exception("Classfile Baz badly formed: no method parameters info");
        if (null == cattr) 
            throw new Exception("Classfile Baz badly formed: no local variable table");
        byte[] bazBytes = ClassFile.of().transformClass(baz, ClassTransform.transformingMethods((methodBuilder, methodElement) -> {
            if (methodElement instanceof MethodParametersAttribute a) {
                List<MethodParameterInfo> newParameterInfos = new ArrayList<>();
                for (MethodParameterInfo info : a.parameters()) {
                    newParameterInfos.add(MethodParameterInfo.ofParameter("baz".describeConstable(), info.flagsMask()));
                }
                a = MethodParametersAttribute.of(newParameterInfos);
                methodBuilder.with(a);
            } else {
                methodBuilder.with(methodElement);
            }
        }));
        if (flip) {
            bazBytes = ClassFile.of().transformClass(baz, ClassTransform.transformingMethods((methodBuilder, methodElement) -> {
                if (methodElement instanceof MethodParametersAttribute) {
                    methodBuilder.with(cattr);
                } else if (methodElement instanceof CodeAttribute) {
                    methodBuilder.with(mpattr);
                } else {
                    methodBuilder.with(methodElement);
                }
            }));
        }
        Files.write(Baz_class.toPath(), bazBytes);
    }
    void checkFoo() throws Exception {
        File Foo_class = new File(classesdir, Foo_name + ".class");
        ClassModel foo = ClassFile.of().parse(Foo_class.toPath());
        for (int i = 0; i < foo.methods().size(); i++) {
            System.err.println("Examine method Foo." + foo.methods().get(i).methodName());
            if (foo.methods().get(i).methodName().equalsString("foo2")) {
                for (int j = 0; j < foo.methods().get(i).attributes().size(); j++) 
                    if (foo.methods().get(i).attributes().get(j) instanceof MethodParametersAttribute mp) {
                        System.err.println("Foo.foo2 should have 2 parameters: j and k");
                        if (2 != mp.parameters().size()) 
                            error("expected 2 method parameter entries in foo2, got " + mp.parameters().size()); else if (!mp.parameters().get(0).name().orElseThrow().equalsString("j")) 
                            error("expected first parameter to foo2 to be \"j\", got \"" + mp.parameters().get(0).name().orElseThrow().stringValue() + "\" instead"); else if (!mp.parameters().get(1).name().orElseThrow().equalsString("k")) 
                            error("expected first parameter to foo2 to be \"k\", got \"" + mp.parameters().get(1).name().orElseThrow() + "\" instead");
                    }
            } else if (foo.methods().get(i).methodName().equalsString("<init>")) {
                for (int j = 0; j < foo.methods().get(i).attributes().size(); j++) {
                    if (foo.methods().get(i).attributes().get(j) instanceof MethodParametersAttribute) 
                        error("Zero-argument constructor shouldn't have MethodParameters");
                }
            } else if (foo.methods().get(i).methodName().equalsString("foo0")) {
                for (int j = 0; j < foo.methods().get(i).attributes().size(); j++) 
                    if (foo.methods().get(i).attributes().get(j) instanceof MethodParametersAttribute) 
                        error("Zero-argument method shouldn't have MethodParameters");
            } else 
                error("Unknown method " + foo.methods().get(i).methodName() + " showed up in class Foo");
        }
    }
    void checkBar() throws Exception {
        File Bar_class = new File(classesdir, Bar_name + ".class");
        ClassModel bar = ClassFile.of().parse(Bar_class.toPath());
        for (int i = 0; i < bar.methods().size(); i++) {
            System.err.println("Examine method Bar." + bar.methods().get(i).methodName());
            if (bar.methods().get(i).methodName().equalsString("<init>")) {
                for (int j = 0; j < bar.methods().get(i).attributes().size(); j++) 
                    if (bar.methods().get(i).attributes().get(j) instanceof MethodParametersAttribute mp) {
                        System.err.println("Bar constructor should have 1 parameter: i");
                        if (1 != mp.parameters().size()) 
                            error("expected 1 method parameter entries in constructor, got " + mp.parameters().size()); else if (!mp.parameters().get(0).name().orElseThrow().equalsString("i")) 
                            error("expected first parameter to foo2 to be \"i\", got \"" + mp.parameters().get(0).name().orElseThrow() + "\" instead");
                    }
            } else if (bar.methods().get(i).methodName().equalsString("foo")) {
                for (int j = 0; j < bar.methods().get(i).attributes().size(); j++) {
                    if (bar.methods().get(i).attributes().get(j) instanceof MethodParametersAttribute) 
                        error("Zero-argument constructor shouldn't have MethodParameters");
                }
            }
        }
    }
    String compile(String... args) throws Exception {
        System.err.println("compile: " + Arrays.asList(args));
        StringWriter sw = new StringWriter();
        PrintWriter pw = new PrintWriter(sw);
        int rc = com.sun.tools.javac.Main.compile(args, pw);
        pw.close();
        String out = sw.toString();
        if (out.length() > 0) 
            System.err.println(out);
        if (rc != 0) 
            error("compilation failed, rc=" + rc);
        return out;
    }
    File writeFile(File dir, String path, String body) throws IOException {
        File f = new File(dir, path);
        f.getParentFile().mkdirs();
        FileWriter out = new FileWriter(f);
        out.write(body);
        out.close();
        return f;
    }
    void error(String msg) {
        System.err.println("Error: " + msg);
        errors++;
    }
    int errors;
}
