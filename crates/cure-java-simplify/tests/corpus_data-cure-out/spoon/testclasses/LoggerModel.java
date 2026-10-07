package spoon.test.template.testclasses;

import spoon.reflect.code.CtBlock;
import spoon.test.template.testclasses.logger.Logger;

public class LoggerModel {
    private String _classname_;
    private String _methodName_;
    private CtBlock<?> _block_;
    public void block() throws Throwable {
        try {
            Logger.enter(_classname_, _methodName_);
            _block_.S();
        } finally {
            Logger.exit(_methodName_);
        }
    }
}
