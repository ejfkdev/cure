package com . google . googlejavaformat . java . javatests;

import com . google . common . base . Charsets;
import com . google . common
. collect . Range;
import com . google . common . collect . RangeSet;
import com . google . common . collect . TreeRangeSet;
import com . google .
googlejavaformat . java . Formatter;
import com . google . googlejavaformat .
java . JavaCommentsHelper;
import com . google . googlejavaformat . java .
JavaInput;
import com . google . googlejavaformat . java . JavaOutput;
import junit . framework . TestCase;
import java . io . File;
import java . io . StringWriter;
import java . io .
Writer;
import java . nio . file . FileSystems;
import java . nio . file .
Files;
import java . nio . file . Path;
import java . util . ArrayList;
import java . util . List;

public final class Unformatted extends TestCase {
    private static final int MAX_WIDTH = 100;
    public void testFormatter() throws Exception {
        Path inputPath = FileSystems.getDefault().getPath(TestPath.getDir(), "input");
        Path outputPath = FileSystems.getDefault().getPath(TestPath.getDir(), "output");
        File inputDirectory = new File(inputPath.toString());
        File outputDirectory = new File(outputPath.toString());
        for (File file : inputDirectory.listFiles()) {
            assertTrue(file.isFile());
            String fileName = file.getName();
            assertTrue(fileName.endsWith(".java"));
            String inputString = new String(Files.readAllBytes(FileSystems.getDefault().getPath(inputDirectory.toString(), fileName)), Charsets.UTF_8);
            String expectedOutputString = new String(Files.readAllBytes(FileSystems.getDefault().getPath(outputDirectory.toString(), fileName)), Charsets.UTF_8);
            JavaInput javaInput = new JavaInput(inputString);
            JavaOutput javaOutput = new JavaOutput(javaInput, new JavaCommentsHelper(), false);
            List< String > errors = new ArrayList< >();
            Formatter.format(javaInput, javaOutput, MAX_WIDTH, errors, 1);
            assertTrue(errors.isEmpty());
            Writer stringWriter = new StringWriter();
            RangeSet< Integer > linesFlag = TreeRangeSet.create();
            linesFlag.add(Range.all());
            javaOutput.writeMerged(stringWriter, linesFlag, MAX_WIDTH, errors);
            assertEquals(stringWriter.toString(), expectedOutputString);
        }
    }
}
