package com.archieapps.ribalta

import com.archieapps.ribalta.lesson.LessonTarget
import com.archieapps.ribalta.lesson.TargetOption
import com.archieapps.ribalta.lesson.compileLesson
import com.archieapps.ribalta.lesson.targets
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class LessonTargetsTest {
    @Test
    fun compileLessonReturnsDocumentAndDiagnostics() {
        val result = compileLesson("# Titulo\n\nTexto.", 400)
        assertEquals(1, result.document.pages.size)
        assertEquals(400f, result.document.pages[0].width)
        assertEquals(null, result.document.pages[0].height)
        assertTrue(result.diagnostics.none { it.severity == "error" })
    }

    @Test
    fun listsPagesAndAnchorsWithLabels() {
        val document = compileLesson("# Primeira\n\nTexto. {#fase-clara}\n\n--- {#segunda}\n\n# Segunda\n\nOutro texto. {#fase-escura}", 400).document
        val targets = document.targets()
        val anchors = targets.filter { it.target is LessonTarget.Anchor }
        assertEquals(listOf("fase-clara", "fase-escura"), anchors.map { (it.target as LessonTarget.Anchor).id })
        assertEquals(listOf("Texto.", "Outro texto."), anchors.map { it.label })
        assertEquals(listOf("Primeira", "Segunda"), anchors.map { it.pageTitle })
        val pages = targets.filter { it.target is LessonTarget.Page }
        assertEquals(document.pages.map { it.id }, pages.map { (it.target as LessonTarget.Page).id })
        assertEquals(listOf("Primeira", "Segunda"), pages.map { it.label })
    }

    @Test
    fun truncatesLongLabels() {
        val long = "palavra ".repeat(40).trim()
        val document = compileLesson("$long {#longo}", 400).document
        val option: TargetOption = document.targets().first { it.target == LessonTarget.Anchor("longo") }
        assertTrue(option.label.length <= 61)
        assertTrue(option.label.endsWith("…"))
    }

    @Test
    fun pagesWithoutHeadingFallBackToPageId() {
        val document = compileLesson("Texto solto.", 400).document
        val page = document.targets().first { it.target is LessonTarget.Page }
        assertEquals(document.pages[0].id, page.label)
        assertEquals(null, page.pageTitle)
    }

    @Test
    fun listsListItemAnchors() {
        val document = compileLesson("# Lista\n\n- primeiro {#item-um}\n- segundo", 400).document
        val option = document.targets().first { it.target == LessonTarget.Anchor("item-um") }
        assertEquals("primeiro", option.label)
        assertEquals("Lista", option.pageTitle)
    }

    @Test
    fun invalidLessonStillReturnsDocumentAndErrors() {
        val result = compileLesson("# Titulo\n\nTexto. {#a}\n\n:: sound https://exemplo.com/a.mp3", 400)
        assertTrue(result.diagnostics.any { it.severity == "error" })
        assertTrue(result.document.pages.isNotEmpty())
        assertTrue(result.document.targets().any { it.target == LessonTarget.Anchor("a") })
    }

    @Test
    fun rejectsNonPositiveWidth() {
        assertTrue(runCatching { compileLesson("# a", 0) }.exceptionOrNull() is IllegalArgumentException)
    }
}
