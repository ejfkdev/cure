package com.google.googlejavaformat.intellij;

import com.intellij.formatting.service.FormattingNotificationService;
import com.intellij.openapi.project.Project;

class Notifications {
    static final String PARSING_ERROR_NOTIFICATION_GROUP = "google-java-format parsing error";
    static final String PARSING_ERROR_TITLE = PARSING_ERROR_NOTIFICATION_GROUP;
    static String parsingErrorMessage(String filename) {
        return "google-java-format failed. Does " + filename + " have syntax errors?";
    }
    static void displayParsingErrorNotification(Project project, String filename) {
        FormattingNotificationService.getInstance(project).reportError(Notifications.PARSING_ERROR_NOTIFICATION_GROUP, Notifications.PARSING_ERROR_TITLE, Notifications.parsingErrorMessage(filename));
    }
}
