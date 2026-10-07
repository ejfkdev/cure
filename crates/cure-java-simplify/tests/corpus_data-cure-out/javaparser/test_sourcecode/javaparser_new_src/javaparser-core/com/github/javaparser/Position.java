package com.github.javaparser;

import com.github.javaparser.ast.Node;
import static com.github.javaparser.utils.Utils.assertNotNull;

public class Position implements Comparable<Position> {
    public final int line;
    public final int column;
    public static final Position ABSOLUTE_START = new Position(Node.ABSOLUTE_BEGIN_LINE, -1);
    public static final Position ABSOLUTE_END = new Position(Node.ABSOLUTE_END_LINE, -1);
    public static final Position HOME = new Position(1, 1);
    public static final Position UNKNOWN = new Position(0, 0);
    public Position(int line, int column) {
        if (line < Node.ABSOLUTE_END_LINE) {
            throw new IllegalArgumentException("Can't position at line " + line);
        }
        if (column < -1) {
            throw new IllegalArgumentException("Can't position at column " + column);
        }
        this.line = line;
        this.column = column;
    }
    public static Position pos(int line, int column) {
        return new Position(line, column);
    }
    public Position withColumn(int column) {
        return new Position(this.line, column);
    }
    public Position withLine(int line) {
        return new Position(line, this.column);
    }
    public boolean valid() {
        return line > 0 && column > 0;
    }
    public boolean invalid() {
        return !valid();
    }
    public Position orIfInvalid(Position anotherPosition) {
        return valid() ? this : anotherPosition;
    }
    public boolean isAfter(Position position) {
        assertNotNull(position);
        if (position.line == Node.ABSOLUTE_BEGIN_LINE) 
            return true;
        if (line > position.line) {
            return true;
        } else if (line == position.line) {
            return column > position.column;
        }
        return false;
    }
    public boolean isBefore(Position position) {
        assertNotNull(position);
        if (position.line == Node.ABSOLUTE_END_LINE) 
            return true;
        if (line < position.line) {
            return true;
        } else if (line == position.line) {
            return column < position.column;
        }
        return false;
    }
    @Override
	public int compareTo(Position o) {
        assertNotNull(o);
        return isBefore(o) ? -1 : isAfter(o) ? 1 : 0;
    }
    @Override
	public boolean equals(Object o) {
        if (this == o) 
            return true;
        if (o == null || getClass() != o.getClass()) 
            return false;
        Position position = (Position) o;
        return line == position.line && column == position.column;
    }
    @Override
	public int hashCode() {
        return 31 * line + column;
    }
    @Override
	public String toString() {
        return "(line " + line + ",col " + column + ")";
    }
}
