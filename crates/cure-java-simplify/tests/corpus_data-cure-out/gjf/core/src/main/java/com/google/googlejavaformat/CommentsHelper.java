package com.google.googlejavaformat;

import com.google.googlejavaformat.Input.Tok;
import java.util.Optional;
import java.util.regex.Pattern;

public interface CommentsHelper {
    String rewrite(Input.Tok tok, int maxWidth, int column0);
    static Optional<String> reformatParameterComment(Tok tok) {
        if (!tok.isSlashStarComment()) {
            return Optional.empty();
        }
        var match = PARAMETER_COMMENT.matcher(tok.getOriginalText());
        return !match.matches() ? Optional.empty() : Optional.of(String.format("/* %s= */", match.group(1)));
    }
    Pattern PARAMETER_COMMENT = Pattern.compile("/\\*\\s*(\\p{javaJavaIdentifierStart}\\p{javaJavaIdentifierPart}*(\\Q...\\E)?)\\s*=\\s*\\*/");
}
