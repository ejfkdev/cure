void main() {
}
sealed class CompactSealedOuter permits CompactInnerA, CompactInnerB {
}

final class CompactInnerA extends CompactSealedOuter {
}

final class CompactInnerB extends CompactSealedOuter {
}
