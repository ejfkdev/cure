package /*2*/ com/*3*/./*4*/puppycrawl/*5*/./*6*/tools/*7*/./*8*/checkstyle.grammar/*9*/./*10*/comments/*11*/;

public class InputFullOfBlockComments {
    public/*20*/ static String main(String[] args) {
        String line = "/*I'm NOT comment*/blabla";
        String.CASE_INSENSITIVE_ORDER.equals(line);
        for (Integer i : null) {}
        return line;
    }
}
