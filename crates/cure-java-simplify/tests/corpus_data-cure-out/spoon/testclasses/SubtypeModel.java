package spoon.test.ctType.testclasses;

interface List<E> {
}

class ArrayList<E> implements List<E> {
}

class ListOfX extends ArrayList<X> {
}

class ListOfA1<A> extends ArrayList<A> {
}

class ListOfA3<A,B,C> extends ArrayList<B> {
}

public class SubtypeModel<A extends X> {
    void foo() {
        List listRaw = new ArrayList();
        List<Object> listObject = new ArrayList<>();
        List<?> listAll = new ArrayList<>();
        List<X> listX = new ArrayList<>();
        ListOfX listOfX = new ListOfX();
        ListOfA1<X> listOfA1_X = new ListOfA1<>();
        ListOfA3<O<A>,X,O<Y>> listOfA3_X = new ListOfA3<>();
        List<Y> listY = new ArrayList<>();
        List<? extends X> listExtendsX = new ArrayList<>();
        List<? extends Y> listExtendsY = new ArrayList<>();
        List<? super X> listSuperX = new ArrayList<>();
        List<? super Y> listSuperY = new ArrayList<>();
        listRaw = listSuperY;
        listAll = listSuperY;
        listExtendsX = listExtendsY;
        listExtendsY = listRaw;
        listSuperY = listRaw;
    }
}
