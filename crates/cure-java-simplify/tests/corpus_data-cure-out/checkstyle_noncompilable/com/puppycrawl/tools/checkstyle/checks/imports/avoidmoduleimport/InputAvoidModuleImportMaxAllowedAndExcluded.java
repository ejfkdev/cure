package com.puppycrawl.tools.checkstyle.checks.imports.avoidmoduleimport;

import module java.base;
import module java.xml;
import module java.desktop;

public class InputAvoidModuleImportMaxAllowedAndExcluded {
    public void method() {
        int a = 1;
    }
}
