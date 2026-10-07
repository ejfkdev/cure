package spoon.test.model.testclasses;

import java.io.ByteArrayInputStream;

public class ClassWithSuperOutOfModel extends ByteArrayInputStream {
    public ClassWithSuperOutOfModel(byte[] p_buf) {
        super(p_buf);
    }
    public ClassWithSuperOutOfModel(byte[] p_buf, int p_offset, int p_length) {
        super(p_buf, p_offset, p_length);
    }
}
