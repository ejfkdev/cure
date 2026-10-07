import java.io.*;

interface IVariableAssignmentTarget extends Serializable {
    String get();
}

class TESTVariableAssignmentTarget {
    public TESTVariableAssignmentTarget() {}
    public void write(ObjectOutput out) throws IOException {
        IVariableAssignmentTarget res1 = () -> "fu";
        IVariableAssignmentTarget res2 = () -> "bar";
        out.writeObject(res1);
        out.writeObject(res2);
    }
    public void readCheck(ObjectInput in) throws Exception {
        Object val = ((IVariableAssignmentTarget) in.readObject()).get();
        if (!val.equals("fu")) {
            throw new IllegalArgumentException("Expected 'fu'");
        }
        val = ((IVariableAssignmentTarget) in.readObject()).get();
        if (!val.equals("bar")) {
            throw new IllegalArgumentException("Expected 'bar'");
        }
    }
}
