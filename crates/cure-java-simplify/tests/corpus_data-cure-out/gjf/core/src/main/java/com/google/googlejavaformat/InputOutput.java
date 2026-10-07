package com.google.googlejavaformat;

import com.google.common.collect.DiscreteDomain;
import com.google.common.collect.ImmutableList;
import com.google.common.collect.Range;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public abstract class InputOutput {
    private ImmutableList<String> lines = ImmutableList.of();
    protected static final Range<Integer> EMPTY_RANGE = Range.closedOpen(-1, -1);
    private static final DiscreteDomain<Integer> INTEGERS = DiscreteDomain.integers();
    protected final void setLines(ImmutableList<String> lines) {
        this.lines = lines;
    }
    public final int getLineCount() {
        return lines.size();
    }
    public final String getLine(int lineI) {
        return lines.get(lineI);
    }
    protected final List<Range<Integer>> ranges = new ArrayList<>();
    private static void addToRanges(List<Range<Integer>> ranges, int i, int k) {
        while (ranges.size() <= i) {
            ranges.add(EMPTY_RANGE);
        }
        Range<Integer> oldValue = ranges.get(i);
        ranges.set(i, Range.closedOpen(oldValue.isEmpty() ? k : oldValue.lowerEndpoint(), k + 1));
    }
    protected final void computeRanges(List<? extends Input.Tok> toks) {
        int lineI = 0;
        for (Input.Tok tok : toks) {
            String txt = tok.getOriginalText();
            int lineI0 = lineI;
            lineI += Newlines.count(txt);
            int k = tok.getIndex();
            if (k >= 0) {
                for (int i = lineI0; i <= lineI; i++) {
                    addToRanges(ranges, i, k);
                }
            }
        }
    }
    public static Map<Integer, Range<Integer>> makeKToIJ(InputOutput put) {
        Map<Integer, Range<Integer>> map = new HashMap<>();
        int ijN = put.getLineCount();
        for (int ij = 0; ij <= ijN; ij++) {
            Range<Integer> range = put.getRanges(ij).canonical(INTEGERS);
            for (int k = range.lowerEndpoint(); k < range.upperEndpoint(); k++) {
                if (map.containsKey(k)) {
                    map.put(k, Range.closedOpen(map.get(k).lowerEndpoint(), ij + 1));
                } else {
                    map.put(k, Range.closedOpen(ij, ij + 1));
                }
            }
        }
        return map;
    }
    public final Range<Integer> getRanges(int lineI) {
        return 0 <= lineI && lineI < ranges.size() ? ranges.get(lineI) : EMPTY_RANGE;
    }
    @Override
  public String toString() {
        return "InputOutput{lines=" + lines + ", ranges=" + ranges + '}';
    }
}
