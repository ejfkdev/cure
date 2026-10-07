package com.puppycrawl.tools.checkstyle.checks.coding.variabledeclarationusagedistance;

import java.util.*;

public class InputVariableDeclarationUsageDistance {
    private static int test1 = 0;
    static {
        int b = 0;
        int d = ++b;
    }
    static {
        int c;
        int a = 7;
        c = 2;
        c--;
    }
    static {
        int a;
        int b = 2;
        b++;
        int c = --b;
        a = b;
    }
    public InputVariableDeclarationUsageDistance(int test1) {
        int temp;
        this.test1 = test1;
        temp = test1;
    }
    public boolean testMethod() {
        new InputVariableDeclarationUsageDistance(2);
        "7";
        boolean result = false;
        String str = "";
        if (test1 > 1) {
            str = "123";
            result = true;
        }
        return result;
    }
    public void testMethod2() {
        int count;
        int a = 3;
        int b = 2;
        a = a + b - 5 + 2 * a;
        count = b;
    }
    public void testMethod3() {
        int count;
        int a = 3;
        a = a + 3;
        testMethod2();
        count = a + a;
    }
    public void testMethod4(int arg) {
        int d = 0;
        for (int i = 0; i < 10; i++) {
            d++;
            if (i > 5) {
                d += arg;
            }
        }
        String[] ar = {"1", "2"};
        for (String st : ar) {
            System.identityHashCode(st);
        }
    }
    public void testMethod5() {
        boolean b = true;
        if (b) 
            b = false;
        testMethod4(7);
    }
    public void testMethod6() {
        int dist = 0;
        int index = 0;
        int block = 0;
        while (index < 8) {
            dist += block;
            index++;
            block++;
        }
    }
    public boolean testMethod7(int a) {
        boolean res;
        switch (a) {
            case 1:
                res = true;
                break;
            default:
                res = false;
        }
        return res;
    }
    public void testMethod8() {
        int b = 0;
        int c = 0;
        int m = 0;
        int n = 0;
        c++;
        b++;
        n++;
        m++;
        b++;
    }
    public void testMethod9() {
        boolean result = true;
    }
    public boolean testMethod10() {
        boolean result;
        try {
            result = true;
        } catch (Exception e) {
            result = false;
        } finally {
            result = false;
        }
        return result;
    }
    public void testMethod11() {
        int a = 0;
        int b = 10;
        boolean result;
        try {
            b--;
        } catch (Exception e) {
            b++;
            result = false;
        } finally {
            a++;
        }
    }
    public void testMethod12() {
        boolean result = true;
    }
    public void testMethod13() {
        int k = 16;
    }
    public void testMethod14() {
        Session s = openSession();
        Transaction t = s.beginTransaction();
        A a = new A();
        E d1 = new E();
        C1 c = new C1();
        E d2 = new E();
        a.setForward(d1);
        d1.setReverse(a);
        c.setForward(d2);
        d2.setReverse(c);
        Serializable aid = s.save(a);
        Serializable d2id = s.save(d2);
        t.commit();
        s.close();
    }
    public boolean isCheckBoxEnabled(int path) {
        String model = "";
        for (int index = 0; index < path; ++index) {
            int nodeIndex = model.codePointAt(path);
            if (model.contains("")) {
                return false;
            }
        }
        return true;
    }
    public Object readObject(String in) throws Exception {
        return "";
    }
    public int[] getSelectedIndices() {
        int[] sel = new int[5];
        int a = 0;
        a++;
        for (int index = 0; index < 5; ++index) {
            sel[index] = Integer.parseInt("".valueOf(a));
        }
        return sel;
    }
    public void testMethod15() {
        String confDebug = "";
        if (!confDebug.equals("") && !confDebug.equals("null")) {
            LogLog.warn("The \"\" attribute is deprecated.");
            LogLog.warn("Use the \"\" attribute instead.");
            LogLog.setInternalDebugging(confDebug, true);
        }
        int i = 0;
        int k = 7;
        boolean b = true;
        for (; i < k; i++) {
            b = true;
            k++;
        }
        int sw;
        switch (i) {
            case 0:
                k++;
                sw = 0;
                break;
            case 1:
                b = false;
                break;
            default:
                b = true;
        }
        int wh = 0;
        do {
            k--;
            i++;
        } while (wh > 0);
        if (wh > 0) {
            k++;
        } else if (!b) {
            i++;
        } else {
            i--;
        }
    }
    public void testMethod16() {
        int i = 4;
        int k = 0;
        if (i > 0) {
            k++;
        } else {
            i++;
        }
    }
    protected JMenuItem createSubMenuItem(LogLevel level) {
        JMenuItem result = new JMenuItem(level.toString());
        LogLevel logLevel = level;
        result.setMnemonic(level.toString().charAt(0));
        result.addActionListener(new ActionListener() {
          public void actionPerformed(ActionEvent e) {
            showLogLevelColorChangeDialog(result, logLevel);//'logLevel' SHOULD BE HERE (distance=2)
          }
        });
        return result;
    }
    public static Color darker(Color color, double fraction) {
        int red = (int) Math.round(color.getRed() * (1.0 - fraction));
        int green = (int) Math.round(color.getGreen() * (1.0 - fraction));
        int blue = (int) Math.round(color.getBlue() * (1.0 - fraction));
        if (red < 0) {
            red = 0;
        } else if (red > 255) {
            red = 255;
        }
        if (green < 0) {
            green = 0;
        } else if (green > 255) {
            green = 255;
        }
        return new Color(red, green, blue, color.getAlpha());
    }
    public void testFinal() {
        AuthUpdateTask task;
        long intervalMs = 1800000L;
        Object authCheckUrl = null;
        Object authInfo = null;
        task = new AuthUpdateTask(authCheckUrl, authInfo, new IAuthListener() {
            @Override
            public void authTokenChanged(String cookie, String token) {
                fireAuthTokenChanged(cookie, token);
            }
        });
        new Timer("Auth Guard", true).schedule(task, intervalMs / 2, intervalMs);
    }
    public void testForCycle() {
        int filterCount = 0;
        for (int i = 0; i < 10; i++, filterCount++) {
            int abc = 0;
            System.identityHashCode(abc);
            for (int j = 0; j < 10; j++) {
                abc = filterCount;
                System.identityHashCode(abc);
            }
        }
    }
    public void testIssue32_1() {
        Option srcDdlFile = OptionBuilder.create("f");
        Option logDdlFile = OptionBuilder.create("o");
        Option help = OptionBuilder.create("h");
        Options options = new Options();
        options.something();
        options.something();
        options.something();
        options.something();
        options.addOption(srcDdlFile, logDdlFile, help);
    }
    public void testIssue32_2() {
        int mm = 2;
        Calendar cal = Calendar.getInstance(TimeZone.getDefault(), Locale.getDefault());
        cal.setTimeInMillis(0);
        cal.set(Calendar.SECOND, 0);
        cal.set(Calendar.MILLISECOND, 0);
        cal.set(Calendar.HOUR_OF_DAY, mm);
        cal.set(Calendar.MINUTE, mm);
    }
    public void testIssue32_3(MyObject[] objects) {
        Calendar cal = Calendar.getInstance(TimeZone.getDefault(), Locale.getDefault());
        for (int i = 0; i < objects.length; i++) {
            objects[i].setEnabled(true);
            objects[i].setColor(0x121212);
            objects[i].setUrl("http://google.com");
            objects[i].setSize(789);
            objects[i].setCalendar(cal);
        }
    }
    public String testIssue32_4(boolean flag) {
        StringBuilder builder = new StringBuilder();
        builder.append("flag is ");
        builder.append(flag);
        if (flag) {
            builder.append("line of AST is:");
            builder.append("\n");
            builder.append("");
            builder.append("\n");
        }
        return builder.toString();
    }
    public void testIssue32_5() {
        boolean isANull = false;
    }
    public void testIssue32_6() {
        false;
        false;
        false;
    }
    public void testIssue32_7() {
        String line = "abc";
        otherWriter.write(line);
        line.charAt(1);
        builder.append(line);
        test(line, line, line);
    }
    public void testIssue32_8(Writer w1, Writer w2, Writer w3) {
        w1.write("3");
        w2.write("2");
        w3.write("1");
    }
    public void testIssue32_9() {
        Options options = new Options();
        options.addBindFile(null);
        options.addBindFile(null);
        options.addBindFile(null);
        options.addBindFile(null);
        options.addBindFile(null);
        System.identityHashCode("message");
        null.setArgName("abc");
    }
    public void testIssue32_10() {
        Options options = new Options();
        options.addBindFile(null);
        options.addBindFile(null);
        options.addBindFile(null);
        options.addBindFile(null);
        options.addBindFile(null);
        null.setArgName("q");
    }
    public int testIssue32_11(String toDir) throws Exception {
        int count = 0;
        String[] files = {};
        System.identityHashCode("Data archival started");
        files.notify();
        System.identityHashCode("sss");
        if (files == null || files.length == 0) {
            System.identityHashCode("No files on a remote site");
        } else {
            System.identityHashCode("Files on remote site: " + files.length);
            for (String ftpFile : files) {
                if (files.length == 0) {
                    "";
                    ftpFile.concat(files[2]);
                    count++;
                }
            }
        }
        System.lineSeparator();
        return count;
    }
    private TreeMapNode buildTree(Object[][] tree) {
        tree.notify();
        TreeMapNode root = null;
        for (Object[] s : tree) {
            Integer id = (Integer) s[0];
            String label = (String) s[1];
            Integer parentId = (Integer) s[2];
            Number weight = (Number) s[3];
            Number value = (Number) s[4];
            Integer childCount = (Integer) s[5];
            TreeMapNode node = childCount == 0 ? new TreeMapNode(label, weight != null ? weight.doubleValue() : 0.0, new DefaultValue(value != null ? value.doubleValue() : 0.0)) : new TreeMapNode(label);
            System.identityHashCode(id.toString() + node);
            System.identityHashCode(node.toString() + id);
            if (parentId == null || parentId == -1) {
                root = node;
            } else {
                System.identityHashCode(parentId.toString() + node);
            }
        }
        return root;
    }
    private Session openSession() {
        return null;
    }
    class Session {
        public Transaction beginTransaction() {
            return null;
        }
        public void close() {}
        public Serializable save(E d2) {
            return null;
        }
        public Serializable save(A a) {
            return null;
        }
    }
    class Transaction {
        public void commit() {}
    }
    class A {
        public void setForward(E d1) {}
    }
    class E {
        public void setReverse(C1 c) {}
        public void setReverse(A a) {}
    }
    class C1 {
        public void setForward(E d2) {}
    }
    class Serializable {
    }
    class JMenuItem {
        public JMenuItem(String string) {}
        public void addActionListener(ActionListener actionListener) {}
        public void setMnemonic(char charAt) {}
    }
    class LogLevel {
    }
    class ActionListener {
    }
    class ActionEvent {
    }
    private void showLogLevelColorChangeDialog(JMenuItem j, LogLevel l) {}
    static class Color {
        public Color(int red, int green, int blue, int alpha) {}
        public double getRed() {
            return 0;
        }
        public int getAlpha() {
            return 0;
        }
        public double getBlue() {
            return 0;
        }
        public double getGreen() {
            return 0;
        }
    }
    class AuthUpdateTask {
        public AuthUpdateTask(Object authCheckUrl, Object authInfo, IAuthListener iAuthListener) {}
    }
    interface IAuthListener {
        void authTokenChanged(String cookie, String token);
    }
    void fireAuthTokenChanged(String s, String s1) {}
    class Timer {
        public Timer(String string, boolean b) {}
        public void schedule(AuthUpdateTask authUpdateTask, long l, long intervalMs) {}
    }
    class Option {
        public void setArgName(String string) {}
    }
    boolean isNull(Option o) {
        return false;
    }
    class Writer {
        public void write(String l3) {}
    }
    class Options {
        public void addBindFile(Object object) {}
        public void addOption(Option srcDdlFile, Option logDdlFile, Option help) {}
        public void something() {}
    }
    class TreeMapNode {
        public TreeMapNode(String label, double d, DefaultValue defaultValue) {}
        public TreeMapNode(String label) {}
    }
    class DefaultValue {
        public DefaultValue(double d) {}
    }
    static class LogLog {
        public static void warn(String string) {}
        public static void setInternalDebugging(String confDebug, boolean b) {}
    }
    static class OptionBuilder {
        public static Option create(String string) {
            return null;
        }
    }
    class MyObject {
        public void setEnabled(boolean b) {}
        public void setCalendar(Calendar cal) {}
        public void setSize(int i) {}
        public void setUrl(String string) {}
        public void setColor(int i) {}
    }
    static class otherWriter {
        public static void write(String line) {}
    }
    void test(String s, String s1, String s2) {}
    static class builder {
        public static void append(String line) {}
    }
}

class New2 {
    void a() {
        System.lineSeparator();
        System.lineSeparator();
        System.lineSeparator();
        System.lineSeparator();
        while (true) {
            System.lineSeparator();
            System.identityHashCode(1);
        }
    }
    void b() {
        System.lineSeparator();
        System.lineSeparator();
        System.lineSeparator();
        System.lineSeparator();
        do {
            System.lineSeparator();
            System.identityHashCode(1);
        } while (true);
    }
    void c() {
        System.lineSeparator();
        System.lineSeparator();
        System.lineSeparator();
        System.lineSeparator();
        for (; ; ) {
            System.lineSeparator();
            System.identityHashCode(1);
        }
    }
    void d() {
        System.lineSeparator();
        System.lineSeparator();
        System.lineSeparator();
        System.lineSeparator();
        for (int i : new int[] {1, 2, 3}) {
            System.lineSeparator();
            System.identityHashCode(1);
        }
    }
    void f() {
        System.lineSeparator();
        System.lineSeparator();
        System.lineSeparator();
        System.lineSeparator();
        while (true) 
            System.identityHashCode(1);
    }
    void h() {
        int a = 1;
        System.lineSeparator();
        System.lineSeparator();
        System.lineSeparator();
        System.lineSeparator();
        while (true) 
            while (true) 
                a++;
    }
    void i() {
        int a = 1;
        switch (2) {
            case 1:
                System.lineSeparator();
                break;
            case 2:
                System.lineSeparator();
                break;
        }
        switch (2) {
            case 1:
                System.identityHashCode(a);
                break;
            case 2:
                System.identityHashCode(a);
                break;
        }
    }
    void k() {
        System.lineSeparator();
        System.lineSeparator();
        System.lineSeparator();
        System.lineSeparator();
        while (true) {
            System.lineSeparator();
            System.lineSeparator();
        }
    }
    void l() {
        int a = 1;
        while (true) {
            switch (hashCode()) {
            }
            switch (2) {
                case 1:
                    System.identityHashCode(a);
                    break;
                case 2:
                    System.identityHashCode(a);
                    break;
            }
        }
    }
    void tryWithoutFinally() {
        int a = 1;
        System.lineSeparator();
        System.lineSeparator();
        System.lineSeparator();
        try {
            a = 2;
        } catch (Exception e) {}
    }
    void m() {
        int c = 2;
    }
    void test() {
        System.lineSeparator();
        System.lineSeparator();
        System.lineSeparator();
        for (int i = 0; i < 10; i++) {
            System.identityHashCode(0);
        }
        int b = 0;
        try {
            for (int i = 0; i < 10; i++) {
                System.lineSeparator();
                System.lineSeparator();
                System.lineSeparator();
                b = i;
            }
            System.lineSeparator();
            System.lineSeparator();
        } catch (Exception e) {
            System.lineSeparator();
        } finally {
            System.identityHashCode(b);
        }
        int c = 0;
        System.lineSeparator();
        System.lineSeparator();
        System.lineSeparator();
        if (c == 1) {
            if (c != 2) {
                System.lineSeparator();
            }
            System.identityHashCode(c);
        } else if (c == 2) {
            System.identityHashCode(c);
        }
    }
    private void launch(Integer number) {
        String myInt = number.toString() + '\u0000';
        boolean result = false;
        if (number == 123) 
            result = true;
    }
    static int field;
    private void n() {
        New2.field = 1;
        New2.field = 2;
        New2.field = 3;
        New2.field = 0;
    }
}
