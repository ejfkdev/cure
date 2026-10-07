package com.puppycrawl.tools.checkstyle.grammar.java19;

import java.util.Iterator;
import java.util.List;

public class InputJava19RecordDecompositionWithConditionInLoops {
    record R(R r) {
    }
    public static void main(String... args) {
        R r = new R(new R(null));
        if (r instanceof R(R recordComponent) r1) {
            System.out.println(recordComponent);
        } else if (r instanceof R(R(R innerRecordComponent) r2) r3) {
            System.out.println(innerRecordComponent);
        } else if (r instanceof R(R(R(R innerInnerRecordComponent)))) {
            System.out.println(innerInnerRecordComponent);
        } else if (r instanceof R(R(R(R(R innerInnerInnerRecordComponent))))) {
            System.out.println(innerInnerInnerRecordComponent);
        } else if (r instanceof R(R(R(R(R(R innerInnerInnerInnerRecordComponent)))))) {
            System.out.println(innerInnerInnerInnerRecordComponent);
        } else if (r instanceof R(R(R(R(R(R(R innerInnerInnerInnerInnerRecordComponent))))))) {
            System.out.println(innerInnerInnerInnerInnerRecordComponent);
        } else if (r instanceof R(R(R(R(R(R(R(R innerInnerInnerInnerInnerInnerRecordComponent)))))))) {
            System.out.println(innerInnerInnerInnerInnerInnerRecordComponent);
        } else if (r instanceof R(R(R(R(R(R(R(R(R innerInnerInnerInnerInnerInnerInnerRecordComponent))))))))) {
            System.out.println(innerInnerInnerInnerInnerInnerInnerRecordComponent);
        } else if (r instanceof R(R myRecord)) {}
        List<R> list = List.of(new R(new R(new R(new R(new R(new R(new R(new R(new R(null))))))))));
        Object o = list.get(0);
        for (; o instanceof R(R rListElement); ) {
            for (; rListElement instanceof R(R rListElementInner) r5; ) {
                for (; rListElementInner instanceof R(R rListElementInnerInner); ) {
                    for (; rListElementInnerInner instanceof R(R rListElementInnerInnerInner); ) {}
                }
            }
        }
        Iterator it = list.iterator();
        while (it.hasNext() && it.next() instanceof R(R rListElement2) r6) {
            while (it.hasNext() && it.next() instanceof R(R rListElementInner2)) {
                while (it.hasNext() && it.next() instanceof R(R rListElementInnerInner2)) {
                    while (it.hasNext() && it.next() instanceof R(R rListElementInnerInnerInner2)) {}
                }
            }
        }
        Object myObj = list.get(0) instanceof R(R myRecord2) r7 && myRecord2 instanceof R(R someComponent) && someComponent instanceof R(R(R someComponent2)) ? myRecord2 : null;
    }
}
