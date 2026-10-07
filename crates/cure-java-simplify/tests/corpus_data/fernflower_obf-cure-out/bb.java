import org.xhtmlrenderer.extend.FSImage;
import org.xhtmlrenderer.extend.ReplacedElement;
import org.xhtmlrenderer.extend.UserAgentCallback;
import org.xhtmlrenderer.layout.LayoutContext;
import org.xhtmlrenderer.pdf.ITextOutputDevice;
import org.xhtmlrenderer.pdf.ITextReplacedElementFactory;
import org.xhtmlrenderer.render.BlockBox;

public class bb extends ITextReplacedElementFactory {
    public static boolean b;
    private static final String[] a;
    public bb(ITextOutputDevice var1) {
        super(var1);
    }
    public ReplacedElement createReplacedElement(LayoutContext param1, BlockBox param2, UserAgentCallback param3, int param4, int param5) {}
    private n<Integer, Integer> a(int param1, int param2, FSImage param3) {}
    static {
        a = new String[] {"img", "src", "type", "code128", "src"};
    }
}
