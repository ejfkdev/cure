module jdk.charsets {
    provides java.nio.charset.spi.CharsetProvider with sun.nio.cs.ext.ExtendedCharsets;
}
