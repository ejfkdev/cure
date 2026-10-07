import java.util.ArrayList;

public class T6294589 {
    private void testme(boolean check) {
        ArrayList<Integer> aList = new ArrayList<Integer>();
        for (Integer i : aList) {
            System.out.println("checking");
        }
        for (Integer i : aList) {
            System.out.println("checking");
        }
    }
}
