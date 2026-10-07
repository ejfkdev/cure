package spoon.test.template.testclasses;

public class TwoSnippets {
    private String bDao;
    private ContextHelper contextHelper;
    public void hello() {
        if (!contextHelper.hasPermission("c")) {
            throw new SecurityException();
        }
        bDao.toString();
    }
    public void toto() {
        if (!contextHelper.hasPermission("c")) {
            throw new SecurityException();
        }
        bDao.toString();
    }
}
