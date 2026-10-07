import javax.xml.parsers.DocumentBuilder;
import javax.xml.parsers.DocumentBuilderFactory;
import javax.xml.parsers.ParserConfigurationException;
import org.w3c.dom.DOMException;
import org.w3c.dom.Document;
import org.w3c.dom.Node;
import org.w3c.dom.ProcessingInstruction;
import org.xml.sax.Attributes;

class az {
    private Document a;
    private Node b;
    private Node c;
    private ay d;
    private static final String[] e;
    public az(ay var1, String var2, String var3, Attributes var4) throws ParserConfigurationException {
        this.d = var1;
        DocumentBuilder var6 = DocumentBuilderFactory.newInstance().newDocumentBuilder();
        this.a = var6.newDocument();
        this.a(var3, var4);
    }
    private boolean a() {
        if (this.b()) {
            this.d.a(new a1(this.b));
            return true;
        }
        this.c = this.c.getParentNode();
        return false;
    }
    private boolean b() {
        return this.c.equals(this.b);
    }
    private void a(String param1, Attributes param2) {}
    public Node c() {
        return this.b;
    }
    public void a(String var1, String var2, Attributes var3) {
        this.a(var2, var3);
    }
    public void a(String var1, String var2) {
        ProcessingInstruction var3 = this.a.createProcessingInstruction(var1, var2);
        this.c.appendChild(var3);
    }
    public boolean b(String var1, String var2) {
        if (!this.c.getNodeName().equals(var2)) {
            throw new DOMException((short) 12, e[0] + var2 + e[1] + this.c.getNodeName());
        }
        return this.a();
    }
    public void a(String var1) {
        this.c.appendChild(this.a.createTextNode(var1));
    }
    public ay d() {
        return this.d;
    }
    static {
        e = new String[] {"Unexpected end-tag: ", " expected: "};
    }
}
