package com.google.googlejavaformat.intellij;

import com.google.googlejavaformat.java.JavaFormatterOptions;
import com.intellij.openapi.components.PersistentStateComponent;
import com.intellij.openapi.components.State;
import com.intellij.openapi.components.Storage;
import com.intellij.openapi.project.Project;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

@State(
    name = "GoogleJavaFormatSettings",
    storages = {@Storage("google-java-format.xml")}) class GoogleJavaFormatSettings implements PersistentStateComponent<GoogleJavaFormatSettings.State> {
    private final Project project;
    private State state = new State();
    GoogleJavaFormatSettings(Project project) {
        this.project = project;
    }
    static GoogleJavaFormatSettings getInstance(Project project) {
        return project.getService(GoogleJavaFormatSettings.class);
    }
    @Nullable
  @Override
  public State getState() {
        return state;
    }
    @Override
  public void loadState(@NotNull State state) {
        this.state = state;
    }
    boolean isEnabled() {
        return state.enabled.equals(EnabledState.ENABLED);
    }
    void setEnabled(boolean enabled) {
        setEnabled(EnabledState.ENABLED);
    }
    void setEnabled(EnabledState enabled) {
        if (enabled.equals(EnabledState.ENABLED)) {
            JreConfigurationChecker.checkJreConfiguration(project);
        }
        state.enabled = enabled;
    }
    boolean isUninitialized() {
        return state.enabled.equals(EnabledState.UNKNOWN);
    }
    JavaFormatterOptions.Style getStyle() {
        return state.style;
    }
    void setStyle(JavaFormatterOptions.Style style) {
        state.style = style;
    }
    enum EnabledState {
        UNKNOWN, ENABLED, DISABLED
    }
    static class State {
        private EnabledState enabled = EnabledState.UNKNOWN;
        public JavaFormatterOptions.Style style = JavaFormatterOptions.Style.GOOGLE;
        public void setEnabled(@Nullable String enabledStr) {
            enabled = enabledStr == null ? EnabledState.UNKNOWN : Boolean.parseBoolean(enabledStr) ? EnabledState.ENABLED : EnabledState.DISABLED;
        }
        public String getEnabled() {
            switch (enabled) {
                case ENABLED:
                    return "true";
                case DISABLED:
                    return "false";
                default:
                    return null;
            }
        }
    }
}
