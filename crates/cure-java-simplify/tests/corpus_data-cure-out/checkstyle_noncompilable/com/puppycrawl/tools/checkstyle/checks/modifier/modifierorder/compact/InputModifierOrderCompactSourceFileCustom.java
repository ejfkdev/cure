void main() {
}

// violation below 'final' modifier out of order with the defined modifier order.
private final void util() {
}

@Deprecated final void annotated() {
}

// violation below ''@Deprecated' annotation modifier.*precede non-annotation modifiers.'
final @Deprecated void wronglyAnnotated() {
}
