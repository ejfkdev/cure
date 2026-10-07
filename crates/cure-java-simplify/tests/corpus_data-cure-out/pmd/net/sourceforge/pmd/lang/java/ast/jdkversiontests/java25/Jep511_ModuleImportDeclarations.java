import module java.base;
import module java.desktop;
import java.util.List;

public class Jep511_ModuleImportDeclarations {
    public static void main(String[] args) {
        File f = new File(".");
        List<File> myList = new ArrayList<>();
        myList.add(f);
        System.out.println("myList = " + myList);
    }
}
