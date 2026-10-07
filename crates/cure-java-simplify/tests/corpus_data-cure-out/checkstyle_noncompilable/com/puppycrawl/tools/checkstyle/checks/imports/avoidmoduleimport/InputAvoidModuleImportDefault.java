package com.puppycrawl.tools.checkstyle.checks.imports.avoidmoduleimport;

import module java.base;
import module java.xml;
import module java.logging;

public class InputAvoidModuleImportDefault {
    public void log() {
        List<String> names = new ArrayList<>();
        names.add("foo");
        DocumentBuilderFactory factory = DocumentBuilderFactory.newInstance();
        try {
            Document document = factory.newDocumentBuilder().newDocument();
            System.out.println(document);
        } catch (ParserConfigurationException e) {
            System.out.println(e.getMessage());
        }
        Logger.getLogger("InputAvoidModuleImportDefault").info("done: " + names);
    }
}
