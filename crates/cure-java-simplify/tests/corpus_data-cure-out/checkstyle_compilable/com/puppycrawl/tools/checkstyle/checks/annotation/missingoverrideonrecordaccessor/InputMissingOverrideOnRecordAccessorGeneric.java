package com.puppycrawl.tools.checkstyle.checks.annotation.missingoverrideonrecordaccessor;

public record InputMissingOverrideOnRecordAccessorGeneric<T>(T value) {
    public T value() {
        return value;
    }
}
