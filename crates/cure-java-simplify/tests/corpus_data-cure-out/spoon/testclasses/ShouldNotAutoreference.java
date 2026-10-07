package spoon.test.imports.testclasses;

import java.nio.file.LinkOption;
import java.util.Arrays;
import java.util.Collection;

public class ShouldNotAutoreference {
    private String toto;
    public static final Collection<LinkOption> NOFOLLOW_LINKS = Arrays.asList(new LinkOption[] {LinkOption.NOFOLLOW_LINKS});
}
