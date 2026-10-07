package com.google.googlejavaformat;

import static com.google.common.truth.Truth.assertThat;
import static org.junit.Assert.fail;
import com.google.common.collect.ImmutableList;
import java.util.Iterator;
import java.util.NoSuchElementException;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

@RunWith(JUnit4.class)
public class NewlinesTest {
    @Test
  public void offsets() {
        assertThat(ImmutableList.copyOf(Newlines.lineOffsetIterator("foo\nbar\n"))).containsExactly(0, 4, 8);
        assertThat(ImmutableList.copyOf(Newlines.lineOffsetIterator("foo\nbar"))).containsExactly(0, 4);
        assertThat(ImmutableList.copyOf(Newlines.lineOffsetIterator("foo\rbar\r"))).containsExactly(0, 4, 8);
        assertThat(ImmutableList.copyOf(Newlines.lineOffsetIterator("foo\rbar"))).containsExactly(0, 4);
        assertThat(ImmutableList.copyOf(Newlines.lineOffsetIterator("foo\r\nbar\r\n"))).containsExactly(0, 5, 10);
        assertThat(ImmutableList.copyOf(Newlines.lineOffsetIterator("foo\r\nbar"))).containsExactly(0, 5);
    }
    @Test
  public void lines() {
        assertThat(ImmutableList.copyOf(Newlines.lineIterator("foo\nbar\n"))).containsExactly("foo\n", "bar\n");
        assertThat(ImmutableList.copyOf(Newlines.lineIterator("foo\nbar"))).containsExactly("foo\n", "bar");
        assertThat(ImmutableList.copyOf(Newlines.lineIterator("foo\rbar\r"))).containsExactly("foo\r", "bar\r");
        assertThat(ImmutableList.copyOf(Newlines.lineIterator("foo\rbar"))).containsExactly("foo\r", "bar");
        assertThat(ImmutableList.copyOf(Newlines.lineIterator("foo\r\nbar\r\n"))).containsExactly("foo\r\n", "bar\r\n");
        assertThat(ImmutableList.copyOf(Newlines.lineIterator("foo\r\nbar"))).containsExactly("foo\r\n", "bar");
    }
    @Test
  public void terminalOffset() {
        Iterator<Integer> it = Newlines.lineOffsetIterator("foo\nbar\n");
        it.next();
        it.next();
        it.next();
        try {
            it.next();
            fail();
        } catch (NoSuchElementException e) {}
        it = Newlines.lineOffsetIterator("foo\nbar");
        it.next();
        it.next();
        try {
            it.next();
            fail();
        } catch (NoSuchElementException e) {}
    }
    @Test
  public void terminalLine() {
        Iterator<String> it = Newlines.lineIterator("foo\nbar\n");
        it.next();
        it.next();
        try {
            it.next();
            fail();
        } catch (NoSuchElementException e) {}
        it = Newlines.lineIterator("foo\nbar");
        it.next();
        it.next();
        try {
            it.next();
            fail();
        } catch (NoSuchElementException e) {}
    }
}
