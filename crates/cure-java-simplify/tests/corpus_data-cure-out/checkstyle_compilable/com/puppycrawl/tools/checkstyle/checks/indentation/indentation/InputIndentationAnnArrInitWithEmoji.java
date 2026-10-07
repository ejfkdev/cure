package com.puppycrawl.tools.checkstyle.checks.indentation.indentation;

@InputIndentationAnnArrInitWithEmoji.Foo({                            //indent:0 exp:0
    @InputIndentationAnnArrInitWithEmoji.Bar,                         //indent:4 exp:0,41,70 warn
@InputIndentationAnnArrInitWithEmoji.Bar,                             //indent:0 exp:0
}) class InputIndentationAnnArrInitWithEmoji {
    @interface Foo {
        Bar[] value() default {};                                         //indent:4 exp:4
        String[] values() default       {   "Hello👍🤩", "Checkstyle", "🎄"};//indent:4 exp:4
    }
    @interface Baz {
        String[] value() default {                                        //indent:4 exp:4
                "🎄",  "Hello",                                                //indent:8 exp:4,29,70 warn
            "Checkstyle"                                                      //indent:4 exp:4
          };                                                                  //indent:2 exp:4 warn
    }
    @interface Bar {
    }
}

interface SomeInterface4 {
    @interface SomeAnnotation {
        String[] values();
    }
    interface Info {
        String A = "🤛🏻a ";
        String B = "b 👇🏻";
    }
    @SomeAnnotation(values =                                            //indent:2 exp:2
      {                                                               //indent:6 exp:2 warn
          "d😆🤛🏻",                                                     //indent:10 exp:2,6,70 warn
            "👆🏻👇🏻",                                                   //indent:12 exp:2,6,70 warn
                    "😂", "  ", "😂🎄",                                  //indent:20 exp:2,6,70 warn
  Info.A,                                                             //indent:2 exp:2
    Info.B                                                            //indent:4 exp:2,6,70 warn
  }                                                                   //indent:2 exp:2
  ) void works();
}
