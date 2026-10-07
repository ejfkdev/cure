package com.puppycrawl.tools.checkstyle.checks.coding.fallthrough;

public class InputFallThrough {
    void tryResource() throws Exception {
        switch (hashCode()) {
            case 1:
                try (Resource resource = new Resource()) {
                    return;
                }
            case 2:
                try (Resource resource = new Resource()) {
                    return;
                } finally {
                    return;
                }
            case 3:
                try (Resource resource = new Resource()) {
                    return;
                } catch (Exception ex) {
                    return;
                }
            case 4:
                try (Resource resource = new Resource()) {} finally {
                    return;
                }
            case 5:
                try (Resource resource = new Resource()) {
                    return;
                }
            case 6:
                try (Resource resource = new Resource()) {} catch (Exception ex) {
                    return;
                }
            case 7:
                try (Resource resource = new Resource()) {}
            case 8:
                try (Resource resource = new Resource()) {}
            case 9:
                try (Resource resource = new Resource()) {} catch (Exception ex) {}
            case 10:
                try (Resource resource = new Resource()) {
                    return;
                } catch (Exception ex) {}
            default:
                break;
        }
    }
    private static class Resource implements AutoCloseable {
        @Override
        public void close() throws Exception {}
    }
}
