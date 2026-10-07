package com.google.googlejavaformat.java;

import static com.google.common.truth.Truth.assertThat;
import com.google.common.collect.Range;
import com.google.common.collect.RangeSet;
import com.google.common.collect.TreeRangeSet;
import java.util.Set;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

@RunWith(JUnit4.class)
public class LineRangesToCharRangesTest {
    @SafeVarargs
  final Set<Range<Integer>> getCharRanges(String input, Range<Integer>... ranges) {
        RangeSet<Integer> rangeSet = TreeRangeSet.create();
        for (Range<Integer> range : ranges) {
            rangeSet.add(range);
        }
        return Formatter.lineRangesToCharRanges(input, rangeSet).asRanges();
    }
    @Test
  public void emptyLineRanges() throws Exception {
        assertThat(getCharRanges("", Range.closedOpen(0, 1))).isEmpty();
    }
    @Test
  public void lineRanges() throws Exception {
        assertThat(getCharRanges("_\n_\n_\n", Range.closedOpen(0, 1))).containsExactly(Range.closedOpen(0, 1));
        assertThat(getCharRanges("_\n_\n_\n", Range.closedOpen(1, 2))).containsExactly(Range.closedOpen(2, 3));
        assertThat(getCharRanges("_\n_\n_\n", Range.closedOpen(2, 3))).containsExactly(Range.closedOpen(4, 5));
        assertThat(getCharRanges("_\n_\n_\n", Range.closedOpen(3, 4))).isEmpty();
    }
    @Test
  public void blankLineRange() throws Exception {
        assertThat(getCharRanges("hello\n\nworld", Range.closedOpen(0, 1))).containsExactly(Range.closedOpen(0, 5));
        assertThat(getCharRanges("hello\n\nworld", Range.closedOpen(1, 2))).isEmpty();
        assertThat(getCharRanges("hello\n\nworld", Range.closedOpen(2, 3))).containsExactly(Range.closedOpen(7, 12));
    }
}
