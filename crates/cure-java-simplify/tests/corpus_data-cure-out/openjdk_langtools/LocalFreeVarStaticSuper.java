class LocalFreeVarStaticSuper {
    static void foo(Object there) {
        class Local {
                    {
                        there.hashCode();
                    }

                    static {
                        class Sub1 extends Local { }
                        class Sub2 extends Local {
                            Sub2() { }
                        }
                        class Sub3 extends Local {
                            Sub3() { super(); }
                        }
                    }

                    static Runnable r = () -> {
                        class Sub1 extends Local { }
                        class Sub2 extends Local {
                            Sub2() { }
                        }
                        class Sub3 extends Local {
                            Sub3() { super(); }
                        }
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
                    class Sub1 extends Local { }
                    class Sub2 extends Local {
                        Sub2() { }
                    }
                    class Sub3 extends Local {
                        Sub3() { super(); }
                    }
                }

                static Runnable r = () -> {
                    class Sub1 extends Local { }
                    class Sub2 extends Local {
                        Sub2() { }
                    }
                    class Sub3 extends Local {
                        Sub3() { super(); }
                    }
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
                            class Sub1 extends Local { }
                            class Sub2 extends Local {
                                Sub2() { }
                            }
                            class Sub3 extends Local {
                                Sub3() { super(); }
                            }
                        }

                        static Runnable r = () -> {
                            class Sub1 extends Local { }
                            class Sub2 extends Local {
                                Sub2() { }
                            }
                            class Sub3 extends Local {
                                Sub3() { super(); }
                            }
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
                        class Sub1 extends Local { }
                        class Sub2 extends Local {
                            Sub2() { }
                        }
                        class Sub3 extends Local {
                            Sub3() { super(); }
                        }
                    }

                    static Runnable r = () -> {
                        class Sub1 extends Local { }
                        class Sub2 extends Local {
                            Sub2() { }
                        }
                        class Sub3 extends Local {
                            Sub3() { super(); }
                        }
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
                        class Sub1 extends Local { }
                        class Sub2 extends Local {
                            Sub2() { }
                        }
                        class Sub3 extends Local {
                            Sub3() { super(); }
                        }
                    }

                    static Runnable r = () -> {
                        class Sub1 extends Local { }
                        class Sub2 extends Local {
                            Sub2() { }
                        }
                        class Sub3 extends Local {
                            Sub3() { super(); }
                        }
                    };
                }
    }
}
