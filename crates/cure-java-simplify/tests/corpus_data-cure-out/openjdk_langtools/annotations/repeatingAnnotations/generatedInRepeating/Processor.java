import java.io.IOException;
import java.io.Writer;
import java.util.Set;
import javax.annotation.processing.AbstractProcessor;
import javax.annotation.processing.RoundEnvironment;
import javax.annotation.processing.SupportedAnnotationTypes;
import javax.lang.model.SourceVersion;
import javax.lang.model.element.TypeElement;

@SupportedAnnotationTypes("*")
public class Processor extends AbstractProcessor {
    int round;
    @Override
    public SourceVersion getSupportedSourceVersion() {
        return SourceVersion.latestSupported();
    }
    @Override
    public boolean process(Set<? extends TypeElement> annotations, RoundEnvironment roundEnv) {
        if (round == 0) {
            try (Writer w = processingEnv.getFiler().createSourceFile("Gen").openWriter()) {
                w.write("class Gen {}");
            } catch (IOException ex) {
                throw new IllegalStateException(ex);
            }
        }
        round++;
        return false;
    }
}
