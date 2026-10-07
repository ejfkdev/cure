import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.*;
import static java.nio.file.StandardCopyOption.REPLACE_EXISTING;

public class Unzip {
    public static void main(String[] args) {
        if (args.length != 1) {
            System.out.println("Usage: Unzip zipfile");
        }
        Path destDir = Paths.get(".");
        try (FileSystem zipFileSystem = FileSystems.newFileSystem(Paths.get(args[0]), null)) {
            Path top = zipFileSystem.getPath("/");
            Files.walk(top).skip(1).forEach((file) -> {
                Path target = destDir.resolve(top.relativize(file).toString());
                System.out.println("Extracting " + target);
                try {
                    Files.copy(file, target, REPLACE_EXISTING);
                } catch (IOException e) {
                    throw new UncheckedIOException(e);
                }
            });
        } catch (UncheckedIOException | IOException e) {
            e.printStackTrace();
            System.exit(1);
        }
    }
}
