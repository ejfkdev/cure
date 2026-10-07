import java.io.*;

interface ITypesOfCapturedArgs extends Serializable {
    Object get();
}

class TESTTypesOfCapturedArgs {
    public TESTTypesOfCapturedArgs() {}
    public void write(ObjectOutput out) throws IOException {
        ITypesOfCapturedArgs res = () -> "hi";
        out.writeObject(res);
    }
    public void readCheck(ObjectInput in) throws Exception {
        Object val = ((ITypesOfCapturedArgs) in.readObject()).get();
        if (!val.equals("hi")) {
            throw new IllegalArgumentException("Expected 'hi'");
        }
    }
}
