package spoon.test.imports.testclasses;

public class StaticImportsFromEnum {
    static enum DataElement {
        KEY("key"), VALUE("value");
        private final String description;
        private DataElement(final String description) {
            this.description = description;
        }
        @Override
        public String toString() {
            return description;
        }
    }
    public DataElement[] getValues() {
        return DataElement.values();
    }
    public ItfWithEnum.Bar[] getBarValues() {
        return ItfWithEnum.Bar.values();
    }
    public ItfWithEnum.Bar getLip() {
        return ItfWithEnum.Bar.Lip;
    }
}
