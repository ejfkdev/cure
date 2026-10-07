class LocalFreeVarStaticInstantiate {
    static void foo(Object there) {
        class Local {
                    {
                        there.hashCode();
                    }

                    static {
                        new Local();    // can't get there from here
                    }

                    static Runnable r = () -> {
                        new Local();    // can't get there from here
                    };
                }
    }
    static Runnable foo = () -> {
    Object there = "";
    class Local {
                {
                    there.hashCode();
                }

                static {
                    new Local();    // can't get there from here
                }

                static Runnable r = () -> {
                    new Local();    // can't get there from here
                };
            }
};
    static Object bar = switch (foo) {
    case Runnable r -> {
        Object there = "";
        class Local {
                        {
                            there.hashCode();
                        }

                        static {
                            new Local();    // can't get there from here
                        }

                        static Runnable r = () -> {
                            new Local();    // can't get there from here
                        };
                    }
                    yield r;
    }
};
    {
        Object there = "";
        class Local {
                    {
                        there.hashCode();
                    }

                    static {
                        new Local();    // can't get there from here
                    }

                    static Runnable r = () -> {
                        new Local();    // can't get there from here
                    };
                }
    }
    static {
        Object there = "";
        class Local {
                    {
                        there.hashCode();
                    }

                    static {
                        new Local();    // can't get there from here
                    }

                    static Runnable r = () -> {
                        new Local();    // can't get there from here
                    };
                }
    }
}
