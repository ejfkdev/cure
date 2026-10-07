import javax.script.ScriptEngine;
import javax.script.ScriptEngineManager;

public class TextBlocks {
    public static void main(String[] args) throws Exception {
        String html = """
                      <html>   
                          <body>
                              <p>Hello, world</p>    
                          </body> 
                      </html>   
                      """;
        System.out.println(html);
        String query = """
                       SELECT `EMP_ID`, `LAST_NAME` FROM `EMPLOYEE_TB`
                       WHERE `CITY` = 'INDIANAPOLIS'
                       ORDER BY `EMP_ID`, `LAST_NAME`;
                       """;
        System.out.println(query);
        ScriptEngine engine = new ScriptEngineManager().getEngineByName("js");
        Object obj = engine.eval("""
                                 function hello() {
                                     print('"Hello, world"');
                                 }
                                 
                                 hello();
                                 """);
        String htmlWithEscapes = """
                      <html>\r
                          <body>\r
                              <p>Hello, world</p>\r
                          </body>\r
                      </html>\r
                      """;
        System.out.println(htmlWithEscapes);
        String season = """
                winter""";
        String period = """
                        winter
                        """;
        String greeting = """
            Hi, "Bob"
            """;
        String salutation = """
            Hi,
             "Bob"
            """;
        String empty = """
                       """;
        String quote = """
                       "
                       """;
        String backslash = """
                           \\
                           """;
        String normalStringLiteral = "test";
        String code = """
            String text = \"""
                A text block inside a text block
            \""";
            """;
        String text = """
                      Lorem ipsum dolor sit amet, consectetur adipiscing \
                      elit, sed do eiusmod tempor incididunt ut labore \
                      et dolore magna aliqua.\
                      """;
        System.out.println(text);
        String colors = """
                        red  \s
                        green\s
                        blue \s
                        """;
        System.out.println(colors);
        String emptyLine = """

test
""";
        System.out.println(emptyLine.replaceAll("\n", "<LF>"));
        String bs = """
                \\test
                """;
        System.out.println(bs.replaceAll("\n", "<LF>"));
        var x = switch (foo) {
            case """
                a label
                """ -> {
                yield """
                            aoeuaoteu
                            """;
            }
        };
    }
}
