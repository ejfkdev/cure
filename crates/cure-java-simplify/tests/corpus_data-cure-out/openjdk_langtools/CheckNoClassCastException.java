import java.util.*;

public class CheckNoClassCastException {
    static String result = "";
    public static void main(String[] args) {
        ListFail.main(null);
        MapFail.main(null);
        if (!result.equals("ListFailDoneMapFailDone")) 
            throw new AssertionError("Incorrect result");
    }
}

class ListFail {
    static interface Foo {
    }
    public static void main(String[] args) {
        List<Date> list = new ArrayList<>();
        list.add(new Date());
        Date date = (Date) ((List<Foo>) (List<?>) list).get(0);
        CheckNoClassCastException.result += "ListFailDone";
    }
}

class MapFail {
    static interface Foo {
    }
    public static void main(String[] args) {
        Map<String,Date> aMap = new HashMap<>();
        aMap.put("test", new Date());
        Date q = (Date) ((Map<String,Foo>) (Map<?,?>) aMap).get("test");
        CheckNoClassCastException.result += "MapFailDone";
    }
}
