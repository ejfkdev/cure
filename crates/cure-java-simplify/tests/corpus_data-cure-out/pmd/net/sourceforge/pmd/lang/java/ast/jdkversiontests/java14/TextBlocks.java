import javax.script.ScriptEngine;
import javax.script.ScriptEngineManager;

public class TextBlocks {
    public static void main(String[] args) throws Exception {
        System.out.println("""
                      <html>   
                          <body>
                              <p>Hello, world</p>    
                          </body> 
                      </html>   
                      """);
        System.out.println("""
                       SELECT `EMP_ID`, `LAST_NAME` FROM `EMPLOYEE_TB`
                       WHERE `CITY` = 'INDIANAPOLIS'
                       ORDER BY `EMP_ID`, `LAST_NAME`;
                       """);
        Object obj = new ScriptEngineManager().getEngineByName("js").eval("""
                                 function hello() {
                                     print('"Hello, world"');
                                 }
                                 
                                 hello();
                                 """);
        System.out.println("""
                      <html>\r
                          <body>\r
                              <p>Hello, world</p>\r
                          </body>\r
                      </html>\r
                      """);
        System.out.println("""
                      Lorem ipsum dolor sit amet, consectetur adipiscing \
                      elit, sed do eiusmod tempor incididunt ut labore \
                      et dolore magna aliqua.\
                      """);
        System.out.println("""
                        red  \s
                        green\s
                        blue \s
                        """);
        System.out.println("""

test
""".replaceAll("\n", "<LF>"));
        System.out.println("""
                \\test
                """.replaceAll("\n", "<LF>"));
    }
}
