package com.puppycrawl.tools.checkstyle.checks.annotation.missingoverrideonrecordaccessor;

public record InputMissingOverrideOnRecordAccessorValid(String name) {
    @Override
    public String name() {
        return name.toUpperCase();
    }
}
