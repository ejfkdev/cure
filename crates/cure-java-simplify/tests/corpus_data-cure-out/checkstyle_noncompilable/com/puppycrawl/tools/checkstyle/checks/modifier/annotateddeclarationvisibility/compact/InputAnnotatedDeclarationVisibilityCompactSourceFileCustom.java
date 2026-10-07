void allowedMethod() {}

// violation below 'Annotated element has disallowed visibility 'private'.'
@Deprecated
@SuppressWarnings("unused")
private void violationPrivateMethod() {}

// violation below 'Annotated element has disallowed visibility 'public'.'
@Deprecated
public void violationPublicMethod() {}

// violation below 'Annotated element has disallowed visibility 'package-private'.'
@Deprecated
void violationPackagePrivate() {}

void main() {}
