package spoon.test.annotation.testclasses.dropwizard;

import spoon.test.annotation.testclasses.PortRange;

public class GraphiteReporterFactory {
    @PortRange
    private int port = 2003;
    public int getPort() {
        return port;
    }
    public void setPort(int port) {
        this.port = port;
    }
}
