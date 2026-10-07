package spoon.test.staticFieldAccess2.testclasses;

@ALong(number = Constants.PRIO)
public class ChildOfConstants extends Constants {
    long p1 = Constants.PRIO;
    long p2 = PRIO;
}
