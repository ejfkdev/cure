package spoon.test.template.testclasses;

public class BServiceImpl {
    private String bDao;
    private ContextHelper contextHelper;
    public void hello() {
        if (!contextHelper.hasPermission("c")) {
            throw new SecurityException();
        }
        bDao.toString();
    }
}
