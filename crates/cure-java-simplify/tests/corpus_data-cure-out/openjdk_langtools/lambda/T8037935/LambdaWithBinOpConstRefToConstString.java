interface MyFI {
    void accept();
}

public class LambdaWithBinOpConstRefToConstString {
    public static void main(String[] args) {
        MyFI consumeStrings = () -> {
            System.out.println(" local constant: mwmwm");
        };
    }
}
