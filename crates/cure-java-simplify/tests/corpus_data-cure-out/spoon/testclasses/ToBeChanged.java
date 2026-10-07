package spoon.test.prettyprinter.testclasses;

import java.util.ArrayList;
import java.util.List;

public
@Deprecated
abstract class ToBeChanged<T, K> extends ArrayList<T /* let's confuse > it */ > implements List<T>, Cloneable {
    final
	//
	private String string = "abc" + "d";
    public <T, K> void andSomeOtherMethod(int param1, String param2, List<?>[][]... twoDArrayOfLists) {
        System.out.println("aaaxyz");
    }
    List<?>[][] twoDArrayOfLists = new List<?>[7][];
}
