public class GitHubBug3642 {
    @interface Foo {
        String v1()[]; // parse error
                // equivalent to String[] v1();
    }
}
