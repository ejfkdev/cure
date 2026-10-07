class Jep441_PatternMatchingForSwitch {
    static String formatterPatternSwitch(Object obj) {
        return switch (obj) {
            case Integer i -> String.format("int %d", i);
            case Long l -> String.format("long %d", l);
            case Double d -> String.format("double %f", d);
            case String s -> String.format("String %s", s);
            default -> obj.toString();
        };
    }
    static void testFooBarNew(String s) {
        switch (s) {
            case null -> System.out.println("Oops");
            case "Foo", "Bar" -> System.out.println("Great");
            default -> System.out.println("Ok");
        }
    }
    static void testStringNew(String response) {
        switch (response) {
            case null -> {}
            case String s
                    when s.equalsIgnoreCase("YES") -> {
                System.out.println("You got it");
            }
            case String s
                    when s.equalsIgnoreCase("NO") -> {
                System.out.println("Shame");
            }
            case String s -> {
                System.out.println("Sorry?");
            }
        }
    }
    static void testStringEnhanced(String response) {
        switch (response) {
            case null -> {}
            case "y", "Y" -> {
                System.out.println("You got it");
            }
            case "n", "N" -> {
                System.out.println("Shame");
            }
            case String s
                    when s.equalsIgnoreCase("YES") -> {
                System.out.println("You got it");
            }
            case String s
                    when s.equalsIgnoreCase("NO") -> {
                System.out.println("Shame");
            }
            case String s -> {
                System.out.println("Sorry?");
            }
        }
    }
    sealed interface CardClassification permits Suit, Tarot {
    }
    public enum Suit implements CardClassification {
        CLUBS, DIAMONDS, HEARTS, SPADES
    }
    final class Tarot implements CardClassification {
    }
    static void exhaustiveSwitchWithoutEnumSupport(CardClassification c) {
        switch (c) {
            case Suit s when s == Suit.CLUBS -> {
                System.out.println("It's clubs");
            }
            case Suit s when s == Suit.DIAMONDS -> {
                System.out.println("It's diamonds");
            }
            case Suit s when s == Suit.HEARTS -> {
                System.out.println("It's hearts");
            }
            case Suit s -> {
                System.out.println("It's spades");
            }
            case Tarot t -> {
                System.out.println("It's a tarot");
            }
        }
    }
    static void exhaustiveSwitchWithBetterEnumSupport(CardClassification c) {
        switch (c) {
            case Suit.CLUBS -> {
                System.out.println("It's clubs");
            }
            case Suit.DIAMONDS -> {
                System.out.println("It's diamonds");
            }
            case Suit.HEARTS -> {
                System.out.println("It's hearts");
            }
            case Suit.SPADES -> {
                System.out.println("It's spades");
            }
            case Tarot t -> {
                System.out.println("It's a tarot");
            }
        }
    }
    sealed interface Currency permits Coin {
    }
    enum Coin implements Currency {
        HEADS, TAILS
    }
    static void goodEnumSwitch1(Currency c) {
        switch (c) {
            case Coin.HEADS -> {
                System.out.println("Heads");
            }
            case Coin.TAILS -> {
                System.out.println("Tails");
            }
        }
    }
    static void goodEnumSwitch2(Coin c) {
        switch (c) {
            case HEADS -> {
                System.out.println("Heads");
            }
            case Coin.TAILS -> {
                System.out.println("Tails");
            }
        }
    }
}
