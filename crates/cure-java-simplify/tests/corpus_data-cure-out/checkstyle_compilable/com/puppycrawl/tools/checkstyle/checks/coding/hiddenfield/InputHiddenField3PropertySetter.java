package com.puppycrawl.tools.checkstyle.checks.coding.hiddenfield;

class InputHiddenField3PropertySetter {
    private int prop;
    public void setProp(int prop) {
        this.prop = prop;
    }
    public void setprop(int prop) {
        this.prop = prop;
    }
    public void setProp(int prop, int extra) {
        this.prop = prop;
    }
}

class PropertySetter23 {
    private int prop;
    public int setProp(int prop) {
        this.prop = prop;
        return 0;
    }
}

class PropertySetter33 {
    private int prop;
    public PropertySetter33 setProp(int prop) {
        this.prop = prop;
        return this;
    }
}

enum PropertySetter43 {
    INSTANCE;
    private int prop;
    private int prop2;
    public void setProp(int prop) {
        this.prop = prop;
    }
    public PropertySetter43 setProp2(int prop2) {
        this.prop2 = prop2;
        return this;
    }
}
