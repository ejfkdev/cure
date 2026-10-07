interface Node {
}

class IntNode implements Node {
    int value;
    IntNode(int value) {
        this.value = value;
    }
}

class NegNode implements Node {
    Node node;
    NegNode(Node node) {
        this.node = node;
    }
}

class MulNode implements Node {
    Node left, right;
    MulNode(Node left, Node right) {
        this.left = left;
        this.right = right;
    }
}

class AddNode implements Node {
    Node left, right;
    AddNode(Node left, Node right) {
        this.left = left;
        this.right = right;
    }
}

public class ExamplesFromProposal {
    public static Object getSomething() {
        return new Long(42);
    }
    public static int eval(Node n) {
        if (n instanceof IntNode in) 
            return in.value; else if (n instanceof NegNode nn) 
            return -eval(nn.node); else if (n instanceof AddNode an) 
            return eval(an.left) + eval(an.right); else if (n instanceof MulNode mn) 
            return eval(mn.left) * eval(mn.right); else {
            throw new AssertionError("broken");
        }
    }
    public static String toString(Node n) {
        if (n instanceof IntNode in) 
            return String.valueOf(in.value); else if (n instanceof NegNode nn) 
            return "-" + eval(nn.node); else if (n instanceof AddNode an) 
            return eval(an.left) + " + " + eval(an.right); else if (n instanceof MulNode mn) 
            return eval(mn.left) + " * " + eval(mn.right); else {
            throw new AssertionError("broken");
        }
    }
    public static Node simplify(Node n) {
        if (n instanceof IntNode in) {
            return n;
        } else if (n instanceof NegNode nn) {
            return new NegNode(simplify(nn.node));
        } else if (n instanceof AddNode ad) {
            n = simplify(ad.left);
            return n instanceof IntNode intn ? intn.value == 0 ? simplify(ad.right) : new AddNode(intn, simplify(ad.right)) : new AddNode(simplify(ad.left), simplify(ad.right));
        } else if (n instanceof MulNode mn) {
            return new MulNode(simplify(mn.left), simplify(mn.right));
        } else {
            throw new AssertionError("broken");
        }
    }
    public static void testNode(Node n, int expected) {
        if (eval(n) != expected) 
            throw new AssertionError("broken");
    }
    public static void main(String[] args) {
        Object x = new Integer(42);
        if (x instanceof Integer i) {
            System.out.println(i.intValue());
        }
        Object obj = getSomething();
        String formatted = "unknown";
        if (obj instanceof Integer i) {
            formatted = String.format("int %d", i);
        } else if (obj instanceof Byte b) {
            formatted = String.format("byte %d", b);
        } else if (obj instanceof Long l) {
            formatted = String.format("long %d", l);
        } else if (obj instanceof Double d) {
            formatted = String.format("double %f", d);
        } else if (obj instanceof String s) {
            formatted = String.format("String %s", s);
        }
        System.out.println(formatted);
        formatted = obj instanceof Integer i ? String.format("int %d", i) : obj instanceof Byte b ? String.format("byte %d", b) : obj instanceof Long l ? String.format("long %d", l) : obj instanceof Double d ? String.format("double %f", d) : obj instanceof String s ? String.format("String %s", s) : String.format("Something else " + obj.toString());
        System.out.println(formatted);
        Node zero = new IntNode(0);
        Node one = new IntNode(1);
        Node ft = new IntNode(42);
        Node temp = new AddNode(zero, ft);
        testNode(temp, 42);
        if (toString(simplify(temp)).equals(toString(ft))) 
            System.out.println("Simplify worked!"); else 
            throw new AssertionError("broken");
        if (toString(simplify(new AddNode(zero, temp))).equals(toString(ft))) 
            System.out.println("Simplify worked!"); else 
            throw new AssertionError("broken");
        temp = new AddNode(zero, ft);
        temp = new AddNode(one, temp);
        temp = new AddNode(zero, temp);
        Node fortythree = new AddNode(one, ft);
        if (toString(simplify(temp)).equals(toString(fortythree))) 
            System.out.println("Simplify worked!"); else 
            throw new AssertionError("broken");
        x = "Hello";
        if (x instanceof String s1) {
            System.out.println(s1);
        }
        if (x instanceof String s1 && s1.length() > 0) {
            System.out.println(s1);
        }
        if (x instanceof String s1) {
            System.out.println(s1 + " is a string");
        } else {
            System.out.println("not a string");
        }
        if (!(x instanceof String s1)) {
            System.out.println("not a string");
        } else {
            System.out.println(s1 + " is a string");
        }
    }
}
