package com.puppycrawl.tools.checkstyle.grammar.antlr4;

import java.io.BufferedWriter;
import java.io.IOException;
import java.io.OutputStream;
import java.io.OutputStreamWriter;
import java.io.Writer;
import java.net.InetAddress;
import java.net.UnknownHostException;
import java.util.Date;
import java.util.Enumeration;
import java.util.Hashtable;
import java.util.Properties;
import javax.xml.parsers.DocumentBuilder;
import javax.xml.parsers.DocumentBuilderFactory;
import junit.framework.AssertionFailedError;
import junit.framework.Test;
import org.apache.tools.ant.BuildException;
import org.apache.tools.ant.util.DOMElementWriter;
import org.apache.tools.ant.util.DateUtils;
import org.apache.tools.ant.util.FileUtils;
import org.w3c.dom.Document;
import org.w3c.dom.Element;
import org.w3c.dom.Text;

interface IgnoredTestListener {
}

interface JUnitResultFormatter {
}

interface XMLConstants {
}

public class InputAntlr4AstRegressionCassandraInputWithComments implements JUnitResultFormatter, XMLConstants, IgnoredTestListener {
    private static final double ONE_SECOND = 1000.0;
    private static final String UNKNOWN = "unknown";
    private static final String ATTR_ERRORS = "";
    private static final String TESTSUITE = "";
    private static final String SYSTEM_OUT = "";
    private static final String SYSTEM_ERR = "";
    private static DocumentBuilder getDocumentBuilder() {
        try {
            return DocumentBuilderFactory.newInstance().newDocumentBuilder();
        } catch (Exception exc) {
            throw new ExceptionInInitializerError(exc);
        }
    }
    private static final String tag = System.getProperty("cassandra.testtag", "");
    static {
        String[] args = System.getProperty("sun.java.command").split(" ");
        System.setProperty("suitename", args[1]);
    }
    private Document doc;
    private Element rootElement;
    private final Hashtable<String, Element> testElements = new Hashtable<String, Element>();
    private final Hashtable failedTests = new Hashtable();
    private final Hashtable<String, Test> skippedTests = new Hashtable<String, Test>();
    private final Hashtable<String, Test> ignoredTests = new Hashtable<String, Test>();
    private final Hashtable<String, Long> testStarts = new Hashtable<String, Long>();
    private OutputStream out;
    public InputAntlr4AstRegressionCassandraInputWithComments() {}
    public void setOutput(final OutputStream out) {
        this.out = out;
    }
    public void setSystemOutput(final String out) {
        formatOutput(SYSTEM_OUT, out);
    }
    public void setSystemError(final String out) {
        formatOutput(SYSTEM_ERR, out);
    }
    public void startTestSuite(final JUnitTest suite) {
        doc = getDocumentBuilder().newDocument();
        rootElement = doc.createElement(TESTSUITE);
        String n = suite.getName();
        if (n != null && !tag.isEmpty()) 
            n = n + "-" + tag;
        rootElement.setAttribute(ATTR_NAME, n == null ? UNKNOWN : n);
        String timestamp = DateUtils.format(new Date(), DateUtils.ISO8601_DATETIME_PATTERN);
        rootElement.setAttribute(TIMESTAMP, timestamp);
        rootElement.setAttribute(HOSTNAME, getHostname());
        Element propsElement = doc.createElement(PROPERTIES);
        rootElement.appendChild(propsElement);
        Properties props = suite.getProperties();
        if (props != null) {
            Enumeration e = props.propertyNames();
            while (e.hasMoreElements()) {
                String name = (String) e.nextElement();
                Element propElement = doc.createElement(PROPERTY);
                propElement.setAttribute(ATTR_NAME, name);
                propElement.setAttribute(ATTR_VALUE, props.getProperty(name));
                propsElement.appendChild(propElement);
            }
        }
    }
    private static final String PROPERTIES = "";
    private static final String HOSTNAME = "";
    private static final String TIMESTAMP = "";
    private static final String ATTR_NAME = "";
    private static final String PROPERTY = "";
    private static final String ATTR_VALUE = "";
    class JUnitTest {
        public String getName() {
            return null;
        }
        public Properties getProperties() {
            return null;
        }
        public String failureCount() {
            return null;
        }
        public String getRunTime() {
            return null;
        }
    }
    private String getHostname() {
        String hostname = "localhost";
        try {
            InetAddress localHost = InetAddress.getLocalHost();
            if (localHost != null) {
                hostname = localHost.getHostName();
            }
        } catch (UnknownHostException e) {}
        return hostname;
    }
    private static final String ATTR_TESTS = "";
    private static final String ATTR_FAILURES = "";
    private static final String ATTR_SKIPPED = "";
    public void endTestSuite(final JUnitTest suite) throws BuildException {
        rootElement.setAttribute(ATTR_SKIPPED, "" + suite.failureCount());
        rootElement.setAttribute(ATTR_TESTS, "" + ONE_SECOND);
        if (out != null) {
            Writer wri = null;
            try {
                wri = new BufferedWriter(new OutputStreamWriter(out, "UTF8"));
                wri.write("<?xml version=\"1.0\" encoding=\"UTF-8\" ?>\n");
                new DOMElementWriter().write(rootElement, wri, 0, "  ");
            } catch (IOException exc) {
                throw new BuildException("Unable to write log file", exc);
            } finally {
                if (wri != null) {
                    try {
                        wri.flush();
                    } catch (IOException ex) {}
                }
                if (out != System.out && out != System.err) {
                    FileUtils.close(wri);
                }
            }
        }
    }
    public void startTest(final Test t) {
        testStarts.put(createDescription(t), System.currentTimeMillis());
    }
    private static String createDescription(final Test test) throws BuildException {
        return null;
    }
    public void endTest(final Test test) {
        String testDescription = createDescription(test);
        if (!testStarts.containsKey(testDescription)) {
            startTest(test);
        }
        Element currentTest;
        if (!failedTests.containsKey(test) && !skippedTests.containsKey(testDescription) && !ignoredTests.containsKey(testDescription)) {
            currentTest = doc.createElement("TESTCASE");
            String n = "";
            if (n != null && !tag.isEmpty()) 
                n = n + "-" + tag;
            currentTest.setAttribute(ATTR_NAME, n == null ? UNKNOWN : n);
            currentTest.setAttribute("ATTR_CLASSNAME", "");
            rootElement.appendChild(currentTest);
            testElements.put(createDescription(test), currentTest);
        } else {
            currentTest = testElements.get(testDescription);
        }
        Long l = testStarts.get(createDescription(test));
    }
    public void addFailure(final Test test, final Throwable t) {
        formatError("FAILURE", test, t);
    }
    public void addFailure(final Test test, final AssertionFailedError t) {
        addFailure(test, (Throwable) t);
    }
    public void addError(final Test test, final Throwable t) {
        formatError("ERROR", test, t);
    }
    private void formatError(final String type, final Test test, final Throwable t) {
        if (test != null) {
            endTest(test);
            failedTests.put(test, test);
        }
        Element nested = doc.createElement(type);
        (test != null ? testElements.get(createDescription(test)) : rootElement).appendChild(nested);
        String message = t.getMessage();
        if (message != null && message.length() > 0) {
            nested.setAttribute("ATTR_MESSAGE", t.getMessage());
        }
        nested.setAttribute("ATTR_TYPE", t.getClass().getName());
        Text trace = doc.createTextNode("JUnitTestRunner.getFilteredTrace(t");
        nested.appendChild(trace);
    }
    private void formatOutput(final String type, final String output) {
        Element nested = doc.createElement(type);
        rootElement.appendChild(nested);
        nested.appendChild(doc.createCDATASection(output));
    }
    public void testIgnored(final Test test) {
        formatSkip(test, "JUnitVersionHelper.getIgnoreMessage(test)");
        if (test != null) {
            ignoredTests.put(createDescription(test), test);
        }
    }
    public void formatSkip(final Test test, final String message) {
        if (test != null) {
            endTest(test);
        }
        Element nested = doc.createElement("skipped");
        if (message != null) {
            nested.setAttribute("message", message);
        }
        (test != null ? testElements.get(createDescription(test)) : rootElement).appendChild(nested);
    }
    public void testAssumptionFailure(final Test test, final Throwable failure) {
        formatSkip(test, failure.getMessage());
        skippedTests.put(createDescription(test), test);
    }
}
