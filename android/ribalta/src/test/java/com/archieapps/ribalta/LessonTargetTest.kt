package com.archieapps.ribalta

import com.archieapps.ribalta.lesson.LessonTarget
import com.archieapps.ribalta.lesson.parse
import com.archieapps.ribalta.lesson.serialize
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class LessonTargetTest {
    @Test
    fun parsesPageAndAnchor() {
        assertEquals(LessonTarget.Page("x"), LessonTarget.parse("page:x"))
        assertEquals(LessonTarget.Anchor("fase-clara"), LessonTarget.parse("anchor:fase-clara"))
    }

    @Test
    fun rejectsInvalidValues() {
        listOf("", "x", "page:", "anchor:", "page", "other:x", "Page:x", "anchor:a b").forEach {
            assertNull(it, LessonTarget.parse(it))
        }
    }

    @Test
    fun serializesWithPrefix() {
        assertEquals("page:x", LessonTarget.Page("x").serialize())
        assertEquals("anchor:y", LessonTarget.Anchor("y").serialize())
    }

    @Test
    fun roundTrips() {
        listOf(LessonTarget.Page("a-1"), LessonTarget.Anchor("b_2")).forEach {
            assertEquals(it, LessonTarget.parse(it.serialize()))
        }
    }
}
