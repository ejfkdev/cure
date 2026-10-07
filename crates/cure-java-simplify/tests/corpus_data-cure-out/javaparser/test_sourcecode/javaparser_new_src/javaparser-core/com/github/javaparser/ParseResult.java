package com.github.javaparser;

import com.github.javaparser.ast.comments.CommentsCollection;
import java.util.List;
import java.util.Optional;
import static com.github.javaparser.utils.Utils.assertNotNull;
import static java.util.Collections.singletonList;
import static com.github.javaparser.utils.Utils.EOL;

public class ParseResult<T> {
    private final Optional<T> result;
    private final List<Problem> problems;
    private final Optional<List<Token>> tokens;
    private final Optional<CommentsCollection> commentsCollection;
    ParseResult(Optional<T> result, List<Problem> problems, Optional<List<Token>> tokens, Optional<CommentsCollection> commentsCollection) {
        this.commentsCollection = assertNotNull(commentsCollection);
        this.result = assertNotNull(result);
        this.problems = assertNotNull(problems);
        this.tokens = assertNotNull(tokens);
    }
    ParseResult(Throwable throwable) {
        this(Optional.empty(), singletonList(new Problem(throwable.getMessage(), Optional.empty(), Optional.of(throwable))), Optional.empty(), Optional.empty());
    }
    public boolean isSuccessful() {
        return problems.isEmpty() && result.isPresent();
    }
    public List<Problem> getProblems() {
        return problems;
    }
    public Optional<List<Token>> getTokens() {
        return tokens;
    }
    public Optional<CommentsCollection> getCommentsCollection() {
        return commentsCollection;
    }
    public Optional<T> getResult() {
        return result;
    }
    @Override
    public String toString() {
        if (isSuccessful()) {
            return "Parsing successful";
        }
        StringBuilder message = new StringBuilder("Parsing failed:").append(EOL);
        for (Problem problem : problems) {
            message.append(problem.toString()).append(EOL);
        }
        return message.toString();
    }
}
