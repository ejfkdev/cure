package com.puppycrawl.tools.checkstyle.checks.coding.unnecessarypermitsclause;

public class InputUnnecessaryPermitsClauseSealedClasses {
}

sealed class SealedA permits SealedAB, SealedAC {
}

final class SealedAB extends SealedA {
}

final class SealedAC extends SealedA {
}

sealed interface SealedI permits SealedIImpl1, SealedIImpl2 {
}

final class SealedIImpl1 implements SealedI {
}

final class SealedIImpl2 implements SealedI {
}

sealed interface SealedInterface permits SealedSubInterface {
}

non-sealed interface SealedSubInterface extends SealedInterface {
}

sealed class SealedOuter1 permits SealedOuter1.InnerA1, SealedOuter1.InnerB1 {
    final class InnerA1 extends SealedOuter1 {
    }
    final class InnerB1 extends SealedOuter1 {
    }
}

sealed interface SealedWithRecord permits RecordChild {
}

record RecordChild() implements SealedWithRecord {
}

sealed interface SealedWithEnum permits EnumChild {
}

enum EnumChild implements SealedWithEnum {
    ONE
}
