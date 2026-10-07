package spoon.test.variable.testclasses;

import java.util.List;
import static spoon.Launcher.SPOONED_CLASSES;

public class MultiBurritos {
    static Object spoon = "bla";
    List<String> Launcher;
    static void toto() {}
    void bar() {
        Launcher.isEmpty();
    }
    void bidule() {
        new Thread(new Runnable() {
			@Override
			public void run() {
				MultiBurritos.toto();
				MultiBurritos.spoon = "truc";
				ForStaticVariables.foo();
			}
		});
    }
}
