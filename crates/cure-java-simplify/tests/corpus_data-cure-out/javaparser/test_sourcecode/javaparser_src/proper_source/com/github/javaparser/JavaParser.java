package com.github.javaparser;

import static com.github.javaparser.PositionUtils.areInOrder;
import static com.github.javaparser.PositionUtils.sortByBeginPosition;
import com.github.javaparser.ASTParser;
import com.github.javaparser.ParseException;
import com.github.javaparser.ast.CompilationUnit;
import com.github.javaparser.ast.ImportDeclaration;
import com.github.javaparser.ast.Node;
import com.github.javaparser.ast.body.BodyDeclaration;
import com.github.javaparser.ast.comments.Comment;
import com.github.javaparser.ast.comments.CommentsCollection;
import com.github.javaparser.ast.comments.CommentsParser;
import com.github.javaparser.ast.comments.LineComment;
import com.github.javaparser.ast.expr.AnnotationExpr;
import com.github.javaparser.ast.expr.Expression;
import com.github.javaparser.ast.stmt.BlockStmt;
import com.github.javaparser.ast.stmt.Statement;
import java.io.File;
import java.io.FileInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.Reader;
import java.io.StringReader;
import java.util.Collections;
import java.util.LinkedList;
import java.util.List;

public final class JavaParser {
    private JavaParser() {}
    private static boolean _doNotAssignCommentsPreceedingEmptyLines = true;
    private static boolean _doNotConsiderAnnotationsAsNodeStartForCodeAttribution = false;
    public static boolean getDoNotConsiderAnnotationsAsNodeStartForCodeAttribution() {
        return _doNotConsiderAnnotationsAsNodeStartForCodeAttribution;
    }
    public static void setDoNotConsiderAnnotationsAsNodeStartForCodeAttribution(boolean doNotConsiderAnnotationsAsNodeStartForCodeAttribution) {
        _doNotConsiderAnnotationsAsNodeStartForCodeAttribution = doNotConsiderAnnotationsAsNodeStartForCodeAttribution;
    }
    public static boolean getDoNotAssignCommentsPreceedingEmptyLines() {
        return _doNotAssignCommentsPreceedingEmptyLines;
    }
    public static void setDoNotAssignCommentsPreceedingEmptyLines(boolean doNotAssignCommentsPreceedingEmptyLines) {
        _doNotAssignCommentsPreceedingEmptyLines = doNotAssignCommentsPreceedingEmptyLines;
    }
    public static CompilationUnit parse(final InputStream in, final String encoding) {
        return parse(in, encoding, true);
    }
    public static CompilationUnit parse(final InputStream in, final String encoding, boolean considerComments) {
        try {
            String code = SourcesHelper.streamToString(in, encoding);
            CompilationUnit cu = new ASTParser(SourcesHelper.stringToStream(code, encoding), encoding).CompilationUnit();
            if (considerComments) {
                insertComments(cu, code);
            }
            return cu;
        } catch (IOException ioe) {
            throw new ParseException(ioe.getMessage());
        }
    }
    public static CompilationUnit parse(final InputStream in) {
        return parse(in, null, true);
    }
    public static CompilationUnit parse(final File file, final String encoding) throws ParseException, IOException {
        return parse(file, encoding, true);
    }
    public static CompilationUnit parse(final File file, final String encoding, boolean considerComments) throws ParseException, IOException {
        FileInputStream in = new FileInputStream(file);
        try {
            return parse(in, encoding, considerComments);
        } finally {
            in.close();
        }
    }
    public static CompilationUnit parse(final File file) throws ParseException, IOException {
        return parse(file, null, true);
    }
    public static CompilationUnit parse(final Reader reader, boolean considerComments) {
        try {
            String code = SourcesHelper.readerToString(reader);
            CompilationUnit cu = new ASTParser(SourcesHelper.stringToReader(code)).CompilationUnit();
            if (considerComments) {
                insertComments(cu, code);
            }
            return cu;
        } catch (IOException ioe) {
            throw new ParseException(ioe.getMessage());
        }
    }
    public static BlockStmt parseBlock(final String blockStatement) {
        StringReader sr = new StringReader(blockStatement);
        BlockStmt result = new ASTParser(sr).Block();
        sr.close();
        return result;
    }
    public static Statement parseStatement(final String statement) {
        StringReader sr = new StringReader(statement);
        Statement stmt = new ASTParser(sr).Statement();
        sr.close();
        return stmt;
    }
    public static ImportDeclaration parseImport(final String importDeclaration) {
        StringReader sr = new StringReader(importDeclaration);
        ImportDeclaration id = new ASTParser(sr).ImportDeclaration();
        sr.close();
        return id;
    }
    public static Expression parseExpression(final String expression) {
        StringReader sr = new StringReader(expression);
        Expression e = new ASTParser(sr).Expression();
        sr.close();
        return e;
    }
    public static AnnotationExpr parseAnnotation(final String annotation) {
        StringReader sr = new StringReader(annotation);
        AnnotationExpr ae = new ASTParser(sr).Annotation();
        sr.close();
        return ae;
    }
    public static BodyDeclaration parseBodyDeclaration(final String body) {
        StringReader sr = new StringReader(body);
        BodyDeclaration bd = new ASTParser(sr).AnnotationBodyDeclaration();
        sr.close();
        return bd;
    }
    private static void insertCommentsInCu(CompilationUnit cu, CommentsCollection commentsCollection) {
        if (commentsCollection.size() == 0) 
            return;
        List<Comment> comments = commentsCollection.getAll();
        PositionUtils.sortByBeginPosition(comments);
        List<Node> children = cu.getChildrenNodes();
        PositionUtils.sortByBeginPosition(children);
        if (cu.getPackage() != null && (children.size() == 0 || PositionUtils.areInOrder(comments.get(0), children.get(0)))) {
            cu.setComment(comments.get(0));
            comments.remove(0);
        }
        insertCommentsInNode(cu, comments);
    }
    private static boolean attributeLineCommentToNodeOrChild(Node node, LineComment lineComment) {
        if (node.getBeginLine() == lineComment.getBeginLine() && !node.hasComment()) {
            node.setComment(lineComment);
            return true;
        } else {
            List<Node> children = new LinkedList<Node>();
            children.addAll(node.getChildrenNodes());
            PositionUtils.sortByBeginPosition(children);
            Collections.reverse(children);
            for (Node child : children) {
                if (attributeLineCommentToNodeOrChild(child, lineComment)) {
                    return true;
                }
            }
            return false;
        }
    }
    private static void insertCommentsInNode(Node node, List<Comment> commentsToAttribute) {
        if (commentsToAttribute.size() == 0) 
            return;
        List<Node> children = node.getChildrenNodes();
        PositionUtils.sortByBeginPosition(children);
        for (Node child : children) {
            List<Comment> commentsInsideChild = new LinkedList<Comment>();
            for (Comment c : commentsToAttribute) {
                if (PositionUtils.nodeContains(child, c, _doNotConsiderAnnotationsAsNodeStartForCodeAttribution)) {
                    commentsInsideChild.add(c);
                }
            }
            commentsToAttribute.removeAll(commentsInsideChild);
            insertCommentsInNode(child, commentsInsideChild);
        }
        List<Comment> attributedComments = new LinkedList<Comment>();
        for (Comment comment : commentsToAttribute) {
            if (comment.isLineComment()) {
                for (Node child : children) {
                    if (child.getEndLine() == comment.getBeginLine()) {
                        if (attributeLineCommentToNodeOrChild(child, comment.asLineComment())) {
                            attributedComments.add(comment);
                        }
                    }
                }
            }
        }
        Comment previousComment = null;
        attributedComments = new LinkedList<Comment>();
        List<Node> childrenAndComments = new LinkedList<Node>();
        childrenAndComments.addAll(children);
        childrenAndComments.addAll(commentsToAttribute);
        PositionUtils.sortByBeginPosition(childrenAndComments, _doNotConsiderAnnotationsAsNodeStartForCodeAttribution);
        for (Node thing : childrenAndComments) {
            if (thing instanceof Comment) {
                previousComment = (Comment) thing;
                if (!previousComment.isOrphan()) {
                    previousComment = null;
                }
            } else {
                if (previousComment != null && !thing.hasComment()) {
                    if (!_doNotAssignCommentsPreceedingEmptyLines || !thereAreLinesBetween(previousComment, thing)) {
                        thing.setComment(previousComment);
                        attributedComments.add(previousComment);
                        previousComment = null;
                    }
                }
            }
        }
        commentsToAttribute.removeAll(attributedComments);
        for (Comment c : commentsToAttribute) {
            if (c.isOrphan()) {
                node.addOrphanComment(c);
            }
        }
    }
    private static boolean thereAreLinesBetween(Node a, Node b) {
        if (!PositionUtils.areInOrder(a, b)) {
            return thereAreLinesBetween(b, a);
        }
        int endOfA = a.getEndLine();
        return b.getBeginLine() > a.getEndLine() + 1;
    }
    private static void insertComments(CompilationUnit cu, String code) throws IOException {
        insertCommentsInCu(cu, new CommentsParser().parse(code));
    }
}
