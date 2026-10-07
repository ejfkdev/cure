package com.google.googlejavaformat.intellij;

import com.google.googlejavaformat.java.FormatterException;
import com.google.googlejavaformat.java.ImportOrderer;
import com.google.googlejavaformat.java.JavaFormatterOptions;
import com.google.googlejavaformat.java.RemoveUnusedImports;
import com.intellij.ide.highlighter.JavaFileType;
import com.intellij.lang.ImportOptimizer;
import com.intellij.openapi.editor.Document;
import com.intellij.openapi.project.Project;
import com.intellij.psi.PsiDocumentManager;
import com.intellij.psi.PsiFile;
import org.jetbrains.annotations.NotNull;

public class GoogleJavaFormatImportOptimizer implements ImportOptimizer {
    @Override
  public boolean supports(@NotNull PsiFile file) {
        return JavaFileType.INSTANCE.equals(file.getFileType()) && GoogleJavaFormatSettings.getInstance(file.getProject()).isEnabled();
    }
    @Override
  public @NotNull Runnable processFile(@NotNull PsiFile file) {
        Project project = file.getProject();
        if (!JreConfigurationChecker.checkJreConfiguration(file.getProject())) {
            return () -> {};
        }
        PsiDocumentManager documentManager = PsiDocumentManager.getInstance(project);
        Document document = documentManager.getDocument(file);
        if (document == null) {
            return () -> {};
        }
        JavaFormatterOptions.Style style = GoogleJavaFormatSettings.getInstance(project).getStyle();
        String origText = document.getText();
        String text;
        try {
            text = ImportOrderer.reorderImports(RemoveUnusedImports.removeUnusedImports(origText), style);
        } catch (FormatterException e) {
            Notifications.displayParsingErrorNotification(project, file.getName());
            return () -> {};
        }
        return text.equals(origText) ? (() -> {}) : (() -> {
            if (documentManager.isDocumentBlockedByPsi(document)) {
                documentManager.doPostponedOperationsAndUnblockDocument(document);
            }
            CharSequence newText = document.getCharsSequence();
            if (CharSequence.compare(origText, newText) != 0) {
                return;
            }
            document.setText(text);
        });
    }
}
