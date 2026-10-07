package com.puppycrawl.tools.checkstyle.checks.design.hideutilityclassconstructor;

import java.awt.Dimension;
import javax.swing.JPanel;

public class InputHideUtilityClassConstructorNonUtilityClass extends JPanel {
    public InputHideUtilityClassConstructorNonUtilityClass() {
        this.setPreferredSize(new Dimension(100, 100));
    }
    public static void utilMethod() {
        System.identityHashCode("I'm a utility method");
    }
}
