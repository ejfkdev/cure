sealed interface SealedTypeChangesIntf permits SealedTypeChanges.A, SealedTypeChangesClass {
}

sealed abstract class SealedTypeChangesCls permits SealedTypeChanges.A, SealedTypeChangesClass {
}

final class SealedTypeChangesClass extends SealedTypeChangesCls implements SealedTypeChangesIntf {
}
