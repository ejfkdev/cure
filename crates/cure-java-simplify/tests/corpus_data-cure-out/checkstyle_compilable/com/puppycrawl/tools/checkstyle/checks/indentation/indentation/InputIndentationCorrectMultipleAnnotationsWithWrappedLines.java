package com.puppycrawl.tools.checkstyle.checks.indentation.indentation;

@WrappedLinesAnnotation1                                                   //indent:0 exp:0
@WrappedLinesAnnotation3(                                                  //indent:0 exp:0
    "value"                                                                //indent:4 exp:4
)                                                                          //indent:0 exp:0
public class InputIndentationCorrectMultipleAnnotationsWithWrappedLines {
    @WrappedLinesAnnotation1                                               //indent:4 exp:4
    @WrappedLinesAnnotation3(                                              //indent:4 exp:4
        "value"                                                            //indent:8 exp:8
    )                                                                      //indent:4 exp:4
    public String value;
    @WrappedLinesAnnotation1                                               //indent:4 exp:4
    @WrappedLinesAnnotation3(                                              //indent:4 exp:4
        "value"                                                            //indent:8 exp:8
    )                                                                      //indent:4 exp:4
    public String method() {
        return "value";
    }
}

@WrappedLinesAnnotation1                                                   //indent:0 exp:0
@WrappedLinesAnnotation3(                                                  //indent:0 exp:0
    "value"                                                                //indent:4 exp:4
) class InputIndentationCorrectMultipleAnnotationsWithWrappedLines2 {
    @WrappedLinesAnnotation1                                               //indent:4 exp:4
    @WrappedLinesAnnotation3(                                              //indent:4 exp:4
        "value"                                                            //indent:8 exp:8
    )                                                                      //indent:4 exp:4
    public String value;
    @WrappedLinesAnnotation1                                               //indent:4 exp:4
    @WrappedLinesAnnotation3(                                              //indent:4 exp:4
        "value"                                                            //indent:8 exp:8
    )                                                                      //indent:4 exp:4
    public String method() {
        return "value";
    }
}

@WrappedLinesAnnotation1                                                   //indent:0 exp:0
@WrappedLinesAnnotation3(                                                  //indent:0 exp:0
    "value"                                                                //indent:4 exp:4
)                                                                          //indent:0 exp:0
@WrappedLinesAnnotation2 class InputIndentationCorrectMultipleAnnotationsWithWrappedLines3 {
    @WrappedLinesAnnotation1                                               //indent:4 exp:4
    @WrappedLinesAnnotation3(                                              //indent:4 exp:4
        "value"                                                            //indent:8 exp:8
    )                                                                      //indent:4 exp:4
    @WrappedLinesAnnotation2                                               //indent:4 exp:4
    public String value;
    @WrappedLinesAnnotation1                                               //indent:4 exp:4
    @WrappedLinesAnnotation3(                                              //indent:4 exp:4
        "value"                                                            //indent:8 exp:8
    )                                                                      //indent:4 exp:4
    @WrappedLinesAnnotation2                                               //indent:4 exp:4
    public String method() {
        return "value";
    }
}

@WrappedLinesAnnotation1                                                   //indent:0 exp:0
@WrappedLinesAnnotation3(                                                  //indent:0 exp:0
    "value"                                                                //indent:4 exp:4
) @WrappedLinesAnnotation2 class InputIndentationCorrectMultipleAnnotationsWithWrappedLines4 {
    @WrappedLinesAnnotation1                                               //indent:4 exp:4
    @WrappedLinesAnnotation3(                                              //indent:4 exp:4
        "value"                                                            //indent:8 exp:8
    ) @WrappedLinesAnnotation2                                             //indent:4 exp:4
    public String value;
    @WrappedLinesAnnotation1                                               //indent:4 exp:4
    @WrappedLinesAnnotation3(                                              //indent:4 exp:4
        "value"                                                            //indent:8 exp:8
    ) @WrappedLinesAnnotation2                                             //indent:4 exp:4
    public String method() {
        return "value";
    }
}

@WrappedLinesAnnotation3(                                                  //indent:0 exp:0
    "value"                                                                //indent:4 exp:4
)                                                                          //indent:0 exp:0
@WrappedLinesAnnotation2 class InputIndentationCorrectMultipleAnnotationsWithWrappedLines5 {
    @WrappedLinesAnnotation3(                                              //indent:4 exp:4
        "value"                                                            //indent:8 exp:8
    )                                                                      //indent:4 exp:4
    @WrappedLinesAnnotation2                                               //indent:4 exp:4
    public String value;
    @WrappedLinesAnnotation3(                                              //indent:4 exp:4
        "value"                                                            //indent:8 exp:8
    )                                                                      //indent:4 exp:4
    @WrappedLinesAnnotation2                                               //indent:4 exp:4
    public String method() {
        return "value";
    }
}
