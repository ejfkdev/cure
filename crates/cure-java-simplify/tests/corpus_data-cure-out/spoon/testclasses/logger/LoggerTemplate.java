package spoon.test.template.testclasses.logger;

import spoon.reflect.code.CtBlock;
import spoon.template.BlockTemplate;
import spoon.template.Local;
import spoon.template.Parameter;

public class LoggerTemplate extends BlockTemplate {
    @Parameter
	private String _classname_;
    @Parameter
	private String _methodName_;
    @Parameter
	private CtBlock<?> _block_;
    @Local
	public LoggerTemplate(String _classname_, String _methodName_, CtBlock<?> _block_) {
        this._classname_ = _classname_;
        this._methodName_ = _methodName_;
        this._block_ = _block_;
    }
    @Override
	public void block() throws Throwable {
        try {
            Logger.enter("_classname_", "_methodName_");
            _block_.S();
        } finally {
            Logger.exit("_methodName_");
        }
    }
}
