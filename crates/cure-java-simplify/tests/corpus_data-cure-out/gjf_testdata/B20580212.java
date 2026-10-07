class B20580212 {
    void m() {
        GroupExpansionReply mockIsgReply = buildRecipientListSubGroupReply(RECIPIENT1, alternatives, alternativesDeltas, false, 0);
        try {} catch (IllegalStateException e) {}
    }
    static class ThrowsAtEndException extends RuntimeException {
    }
}
