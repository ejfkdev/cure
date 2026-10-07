import java.io.*;

interface INameOfCapturedArgs extends Serializable {
    int get();
}

class TESTNameOfCapturedArgs {
    public TESTNameOfCapturedArgs() {}
    public void write(ObjectOutput out) throws IOException {
        INameOfCapturedArgs res = () -> 44;
        out.writeObject(res);
    }
    public void readCheck(ObjectInput in) throws Exception {
        int val = ((INameOfCapturedArgs) in.readObject()).get();
        if (val != 44) {
            throw new IllegalArgumentException("Expected 44");
        }
    }
}
