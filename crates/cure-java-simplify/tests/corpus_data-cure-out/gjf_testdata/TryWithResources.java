class TryWtihResources {
    {
        try (C c = c()) {}
        try (C c = c()) {}
        try (C c = c()) {}
        try (C c = c()) {}
        try (BufferedWriter writer = new BufferedWriter(new OutputStreamWriter(testFile, Charset.defaultCharset()))) {
            writer.append("tom cruise\n").append("avatar\n");
            writer.flush();
        }
        try (Scanner inputScanner = new Scanner(inputStream).useDelimiter("\\s+|,")) {
            while (inputScanner.hasNextLong()) {
                placementIds.add(inputScanner.nextLong());
            }
        }
    }
}
