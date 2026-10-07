package com.puppycrawl.tools.checkstyle.grammar.antlr4;

import static com.puppycrawl.tools.checkstyle.grammar.antlr4.LifecyclePhase.GENERATE_RESOURCES;
import static java.lang.String.format;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Set;
import org.junit.platform.commons.logging.Logger;

class Artifact {
}

class ArtifactRepository {
}

class License {
}

class Model {
}

class Organization {
}

class ProcessRemoteResourcesMojo {
}

@interface Component {
    Class<MavenProjectBuilder> role();
}

class LifecyclePhase {
    public static final Object GENERATE_RESOURCES = null;
}

@interface Mojo {
    String name();
}

@interface Parameter {
    String defaultValue();
    boolean readonly();
}

class ResolutionScope {
    public static final Object RUNTIME = null;
}

class InvalidProjectModelException {
}

class MavenProject {
}

class MavenProjectBuilder {
}

class ProjectBuildingException {
}

@Mojo(name = "")
public class InputAntlr4AstRegressionCommentsOnAnnotationsAndEnums extends ProcessRemoteResourcesMojo {
    public enum ProjectData {
        NONE, LICENSES, FULL
    }
    @Parameter(defaultValue = "NONE", readonly = false)
    protected ProjectData projectsData;
    @Parameter(defaultValue = "${localRepository}", readonly = true)
    private ArtifactRepository localRepositoryThis;
    @Component(role = MavenProjectBuilder.class)
    private MavenProjectBuilder mavenProjectBuilderThis;
    @Parameter(defaultValue = "${project.remoteArtifactRepositories}", readonly = true)
    private List<ArtifactRepository> remoteArtifactRepositoriesThis;
    private Logger getLog() {
        return null;
    }
    enum InputJavadocVariableTagsEnum {
        CONSTANT_A, CONSTANT_B, CONSTANT_C // violation
        {
            /**
             *
             */
            public void someMethod()
            {
            }

            public void someOtherMethod()
            {

            }
        }
    }
}
