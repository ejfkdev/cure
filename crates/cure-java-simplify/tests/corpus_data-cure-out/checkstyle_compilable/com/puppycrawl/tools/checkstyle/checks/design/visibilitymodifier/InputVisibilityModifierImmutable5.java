package com.puppycrawl.tools.checkstyle.checks.design.visibilitymodifier;

import com.google.common.collect.ImmutableSet;
import java.math.BigDecimal;
import java.util.Collection;
import java.util.List;

public final class InputVisibilityModifierImmutable5 {
    public final int someIntValue;
    public final ImmutableSet<String> includes;
    public final ImmutableSet<String> excludes;
    public final String notes;
    public final BigDecimal money;
    public final List list;
    public InputVisibilityModifierImmutable5(Collection<String> includes, Collection<String> excludes, BigDecimal value, String notes, int someValue, List l) {
        this.includes = ImmutableSet.copyOf(includes);
        this.excludes = ImmutableSet.copyOf(excludes);
        this.money = value;
        this.notes = notes;
        this.someIntValue = someValue;
        this.list = l;
    }
    final class Immutable {
        public final float f = 4;
        public final boolean bool = false;
        public final java.net.URI uri = null;
        public final java.io.File file = null;
        public int value = 42;
        public final java.net.URL url = null;
        public boolean rate = false;
        public Long id = 1L;
    }
    class Example {
        final int C_D_E = 0;
    }
}
