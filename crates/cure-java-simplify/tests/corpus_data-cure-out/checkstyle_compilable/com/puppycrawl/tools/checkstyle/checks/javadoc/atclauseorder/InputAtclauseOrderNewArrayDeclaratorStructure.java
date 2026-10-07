package com.puppycrawl.tools.checkstyle.checks.javadoc.atclauseorder;

import java.awt.*;
import java.lang.reflect.AnnotatedElement;
import java.lang.reflect.GenericDeclaration;
import java.lang.reflect.Type;
import java.util.Set;
import javax.transaction.xa.XAException;
import javax.transaction.xa.Xid;

public interface InputAtclauseOrderNewArrayDeclaratorStructure<D extends GenericDeclaration> extends Type, AnnotatedElement {
    Type[] getBounds();
    Xid[] recover(int flag) throws XAException;
}

class Other {
    Set<AWTKeyStroke>[] focusTraversalKeys;
}
