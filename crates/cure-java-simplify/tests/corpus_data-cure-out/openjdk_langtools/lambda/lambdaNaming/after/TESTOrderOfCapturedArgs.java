import java.io.*;

interface IOrderOfCapturedArgs extends Serializable {
    String get();
}

class TESTOrderOfCapturedArgs {
    public TESTOrderOfCapturedArgs() {}
    public void write(ObjectOutput out) throws IOException {
        IOrderOfCapturedArgs res = () -> "barfu";
        out.writeObject(res);
    }
    public void readCheck(ObjectInput in) throws Exception {
        Object val = ((IOrderOfCapturedArgs) in.readObject()).get();
        if (!val.equals("fubar")) {
            throw new IllegalArgumentException("Expected 'fubar'");
        }
    }
}
