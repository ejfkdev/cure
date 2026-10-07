package com.puppycrawl.tools.checkstyle.checks.design.visibilitymodifier;

import org.junit.ClassRule;
import org.junit.Rule;
import org.junit.rules.TemporaryFolder;

public class InputVisibilityModifierAnnotatedSameTypeName {
    @Rule
    public TemporaryFolder rule = new TemporaryFolder();
    @ClassRule
    public TemporaryFolder classRule = new TemporaryFolder();
}
