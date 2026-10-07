import java.io.File;
import java.io.FileInputStream;
import java.io.IOException;
import java.net.URL;
import org.xml.sax.EntityResolver;
import org.xml.sax.InputSource;
import org.xml.sax.SAXException;

class a2 implements EntityResolver {
    final bc a;
    private static final String b;
    a2(bc var1) {
        this.a = var1;
    }
    public InputSource resolveEntity(String var1, String var2) throws SAXException, IOException {
        URL var3 = new URL(var2);
        if (b.equals(var3.getProtocol())) {
            File var4 = new File(var3.getFile());
            if (var4.exists()) {
                return new InputSource(new FileInputStream(var4));
            }
        }
        return null;
    }
    static {
        b = "file";
    }
}
