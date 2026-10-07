package com.puppycrawl.tools.checkstyle.checks.metrics.classfanoutcomplexity;

import com.puppycrawl.tools.checkstyle.checks.metrics.classfanoutcomplexity.inputs.a.aa.AAClass;
import com.puppycrawl.tools.checkstyle.checks.metrics.classfanoutcomplexity.inputs.a.ab.ABClass;
import com.puppycrawl.tools.checkstyle.checks.metrics.classfanoutcomplexity.inputs.b.BClass;
import com.puppycrawl.tools.checkstyle.checks.metrics.classfanoutcomplexity.inputs.c.CClass;

public class InputClassFanOutComplexityExcludedPackagesDirectPackages {
    public AAClass aa;
    public ABClass ab;
    class Inner {
        public BClass b;
        public CClass c;
    }
}

class InputClassFanOutComplexityExcludedPackagesDirectPackagesHidden {
    public CClass c;
}
