package com.google.googlejavaformat.java;

import static com.google.common.truth.Truth.assertThat;
import static org.junit.Assert.assertThrows;
import com.google.common.collect.Range;
import com.google.common.testing.EqualsTester;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.junit.runners.JUnit4;

@RunWith(JUnit4.class)
public class ReplacementTest {
    @Test
  public void testCreateWithValidInput() {
        Replacement replacement = Replacement.create(3, 7, "replacementText");
        assertThat(replacement.replaceRange()).isEqualTo(Range.closedOpen(3, 7));
        assertThat(replacement.replacementString()).isEqualTo("replacementText");
    }
    @Test
  public void invalidReplacementRange() {
        assertThrows(IllegalArgumentException.class, () -> Replacement.create(-1, 5, "text"));
        assertThrows(IllegalArgumentException.class, () -> Replacement.create(10, 5, "text"));
    }
    @Test
  public void testEqualsAndHashCode() {
        new EqualsTester().addEqualityGroup(Replacement.create(0, 4, "abc"), Replacement.create(0, 4, "abc")).addEqualityGroup(Replacement.create(1, 4, "abc")).addEqualityGroup(Replacement.create(0, 4, "def")).testEquals();
    }
}
