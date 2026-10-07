public class StringConversion2 {
    public static void main(String[] args) {
        Object o = "Hello ";
        o += "World!";
        if (!o.equals("Hello World!")) 
            throw new Error("test failed");
    }
}
