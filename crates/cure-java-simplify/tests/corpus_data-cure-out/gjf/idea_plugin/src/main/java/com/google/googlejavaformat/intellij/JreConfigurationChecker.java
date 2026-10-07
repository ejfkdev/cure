package com.google.googlejavaformat.intellij;

import com.google.common.base.Suppliers;
import com.intellij.ide.BrowserUtil;
import com.intellij.notification.Notification;
import com.intellij.notification.NotificationAction;
import com.intellij.notification.NotificationType;
import com.intellij.openapi.actionSystem.AnActionEvent;
import com.intellij.openapi.diagnostic.Logger;
import com.intellij.openapi.project.Project;
import java.util.function.Supplier;
import org.jetbrains.annotations.NotNull;

class JreConfigurationChecker {
    private static final Supplier<Boolean> hasAccess = Suppliers.memoize(JreConfigurationChecker::checkJreConfiguration);
    private static final Logger logger = Logger.getInstance(JreConfigurationChecker.class);
    private final Project project;
    public JreConfigurationChecker(Project project) {
        this.project = project;
    }
    static boolean checkJreConfiguration(Project project) {
        var success = hasAccess.get();
        if (!success) {
            project.getService(JreConfigurationChecker.class).displayConfigurationErrorNotification();
        }
        return success;
    }
    private static boolean checkJreConfiguration() {
        try {
            return testClassAccess("com.sun.tools.javac.api.JavacTrees", "com.sun.tools.javac.code.Flags", "com.sun.tools.javac.file.JavacFileManager", "com.sun.tools.javac.parser.JavacParser", "com.sun.tools.javac.tree.JCTree", "com.sun.tools.javac.util.Log");
        } catch (ClassNotFoundException e) {
            logger.error("Error checking jre configuration for google-java-format", e);
            return false;
        }
    }
    private static boolean testClassAccess(String... classNames) throws ClassNotFoundException {
        for (String className : classNames) {
            if (!testClassAccess(className)) {
                return false;
            }
        }
        return true;
    }
    private static boolean testClassAccess(String className) throws ClassNotFoundException {
        Class<?> klass = Class.forName(className);
        return klass.getModule().isExported(klass.getPackageName(), JreConfigurationChecker.class.getClassLoader().getUnnamedModule());
    }
    private void displayConfigurationErrorNotification() {
        Notification notification = new Notification("Configure JRE for google-java-format", "Configure the JRE for google-java-format", "The google-java-format plugin needs additional configuration before it can be used.", NotificationType.INFORMATION);
        notification.addAction(new NotificationAction("Follow the instructions here") {
          @Override
          public void actionPerformed(
              @NotNull AnActionEvent anActionEvent, @NotNull Notification notification) {
            BrowserUtil.browse(
                "https://github.com/google/google-java-format/blob/master/README.md#intellij-jre-config");
            notification.expire();
          }
        });
        notification.notify(project);
    }
}
