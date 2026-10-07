package com.github.javaparser;

import java.io.IOException;

public interface Provider {
    public int read(char[] buffer, int offset, int len) throws IOException;
    public void close() throws IOException;
}
