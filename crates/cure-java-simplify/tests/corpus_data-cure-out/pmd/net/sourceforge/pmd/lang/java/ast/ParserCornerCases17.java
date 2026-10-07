import java.io.BufferedReader;
import java.io.File;
import java.io.FileReader;
import java.io.IOException;
import java.sql.SQLException;
import java.util.ArrayList;
import java.util.Enumeration;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.zip.ZipEntry;

public class ParserCornerCases17 {
    public ParserCornerCases17() {
        super();
    }
    public void binaryLiterals() {
        int[] phases = {0b00110001, 0b01100010, 0b11000100, 0b10001001, 0b00010011, 0b00100110, 0b01001100, 0b10011000};
        int instruction = 0;
        if ((instruction & 0b11100000) == 0b00000000) {
            switch (instruction & 0b11110000) {
                case 0b00000000:
                    break;
                case 0b00010000:
                    break;
                case 0b00100000:
                    break;
                case 0b00110000:
                    break;
                case 0b01000000:
                    break;
                case 0b01010000:
                    break;
                case 0b01100000:
                    break;
                case 0b01110000:
                    break;
                default:
                    throw new IllegalArgumentException();
            }
        }
    }
    public void underscoreInNumericLiterals() {
        int x10 = 05_2;
    }
    public String stringsInSwitchStatements() {
        String dayOfWeekArg = "Wednesday";
        String typeOfDay;
        switch (dayOfWeekArg) {
            case "Monday":
                typeOfDay = "Start of work week";
                break;
            case "Tuesday":
            case "Wednesday":
            case "Thursday":
                typeOfDay = "Midweek";
                break;
            case "Friday":
                typeOfDay = "End of work week";
                break;
            case "Saturday":
            case "Sunday":
                typeOfDay = "Weekend";
                break;
            default:
                throw new IllegalArgumentException("Invalid day of the week: " + dayOfWeekArg);
        }
        return typeOfDay;
    }
    class MyClass<X> {
        <T> MyClass(T t) {}
    }
    public void typeInferenceForGenericInstanceCreation() {
        Map<String, List<String>> myMap = new HashMap<>();
        List<String> list = new ArrayList<>();
        list.add("A");
        List<? extends String> list2 = new ArrayList<>();
        list.addAll(list2);
        MyClass<Integer> myObject = new MyClass<>("");
    }
    public void theTryWithResourcesStatement() throws IOException {
        try (BufferedReader br = new BufferedReader(new FileReader("/foo"))) {
            String first = br.readLine();
        }
        java.nio.charset.Charset charset = java.nio.charset.Charset.forName("US-ASCII");
        java.nio.file.Path outputFilePath = java.nio.file.Paths.get("/foo-out");
        try (java.util.zip.ZipFile zf = new java.util.zip.ZipFile("/foo.zip"); java.io.BufferedWriter writer = java.nio.file.Files.newBufferedWriter(outputFilePath, charset)) {
            for (Enumeration<? extends ZipEntry> entries = zf.entries(); entries.hasMoreElements(); ) {
                String newLine = System.getProperty("line.separator");
                String zipEntryName = ((java.util.zip.ZipEntry) entries.nextElement()).getName() + newLine;
                writer.write(zipEntryName, 0, zipEntryName.length());
            }
        }
    }
    public void catchingMultipleExceptionTypes() throws IOException, SQLException {
        try {
            if (new File("foo").createNewFile()) {
                throw new SQLException();
            }
        } catch (IOException | SQLException ex) {
            ex.printStackTrace();
            throw ex;
        }
    }
    public void expressionInCastExpression() {
        int initialSizeGlobal = (int) (profilingContext.m_profileItems.size() * (150.0 * 0.30));
    }
}
