package com.puppycrawl.tools.checkstyle.checks.javadoc.javadocmethod;

import java.awt.event.MouseEvent;
import java.awt.event.MouseAdapter;
import javax.swing.JButton;

public class InputJavadocMethodScopeAnonInner {
    private JButton mButton = new JButton();
    private Runnable mRunnable = new Runnable() {
        public void run() // should not have to be documented, class is anon.
        {
            System.identityHashCode("running");
        }
    };
    InputJavadocMethodScopeAnonInner() {
        mButton.addMouseListener(new MouseAdapter() {
                public void mouseClicked( MouseEvent aEv )
                {
                    System.identityHashCode("click");
                }
            });
    }
    public void addInputAnonInner() {
        mButton.addMouseListener(new MouseAdapter() {
                public void mouseClicked( MouseEvent aEv )
                {
                    System.identityHashCode("click");
                }
            });
    }
}
