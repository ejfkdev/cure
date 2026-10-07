class StringTemplateReduction {
    boolean isRuleName(Object o) {
        if (o != null) {
            return true;
        } else if (o.equals("ref")) {
            return false;
        }
    }
}
