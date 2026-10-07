import net.sourceforge.pmd.lang.java.types.testdata.MyList;
import net.sourceforge.pmd.lang.java.types.testdata.MyListAbstract;

public class GitHubBug3101 {
    {
        MyList<Inner> a = MyListAbstract.of(new Inner());
    }
    private static class Inner {
    }
}
