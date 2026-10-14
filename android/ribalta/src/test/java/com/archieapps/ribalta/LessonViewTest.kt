package com.archieapps.ribalta

import android.graphics.Bitmap
import android.graphics.Canvas
import androidx.activity.ComponentActivity
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.dp
import com.archieapps.ribalta.ir.Document
import com.archieapps.ribalta.lesson.LessonTarget
import com.archieapps.ribalta.lesson.LessonView
import com.archieapps.ribalta.lesson.LessonViewState
import com.archieapps.ribalta.lesson.compileLesson
import com.archieapps.ribalta.lesson.rememberLessonViewState
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(NativeSharingRobolectricRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [35], qualifiers = "w400dp-h800dp-mdpi")
class LessonViewTest {
    @get:Rule
    val compose = createAndroidComposeRule<ComponentActivity>()

    private fun longLesson(): String {
        val filler = (1..12).joinToString("\n\n") { "Paragrafo de enchimento numero $it com algum texto para ocupar espaco." }
        return "# Primeira\n\n$filler\n\n--- {#segunda}\n\n# Segunda\n\n$filler\n\n--- {#terceira}\n\n# Terceira\n\nAlvo final. {#alvo}"
    }

    private lateinit var scope: CoroutineScope

    private fun show(source: String): Pair<Document, LessonViewState> {
        val document = compileLesson(source, 400).document
        var state: LessonViewState? = null
        compose.setContent {
            val created = rememberLessonViewState()
            scope = rememberCoroutineScope()
            state = created
            CompositionLocalProvider(LocalDensity provides Density(1f)) {
                Box(Modifier.width(400.dp).height(800.dp)) { LessonView(document, created, Modifier.fillMaxSize()) }
            }
        }
        compose.waitForIdle()
        return document to state!!
    }

    private fun snapshot(): IntArray {
        val bitmap = Bitmap.createBitmap(400, 800, Bitmap.Config.ARGB_8888)
        compose.runOnIdle { compose.activity.window.decorView.draw(Canvas(bitmap)) }
        return IntArray(400 * 800).also { bitmap.getPixels(it, 0, 400, 0, 0, 400, 800) }
    }

    private fun showWithEarlyScroll(source: String, target: LessonTarget): LessonViewState {
        val document = compileLesson(source, 400).document
        var state: LessonViewState? = null
        compose.mainClock.autoAdvance = false
        compose.setContent {
            val created = rememberLessonViewState()
            state = created
            LaunchedEffect(Unit) { created.scrollTo(target) }
            CompositionLocalProvider(LocalDensity provides Density(1f)) {
                Box(Modifier.width(400.dp).height(800.dp)) { LessonView(document, created, Modifier.fillMaxSize()) }
            }
        }
        compose.mainClock.advanceTimeBy(1_000)
        compose.waitForIdle()
        return state!!
    }

    private fun scrollTo(state: LessonViewState, target: LessonTarget) {
        compose.mainClock.autoAdvance = false
        compose.runOnUiThread { scope.launch { state.scrollTo(target) } }
        compose.mainClock.advanceTimeBy(1_000)
        compose.waitForIdle()
    }

    @Test
    fun stacksAllPagesAndReachesEndWhenContentFits() {
        val (document, state) = show("# Um\n\nTexto. {#a}\n\n--- {#b}\n\n# Dois\n\nOutro.")
        assertEquals(2, document.pages.size)
        assertTrue(state.reachedEnd)
        assertTrue(state.contentHeight > 0)
    }

    @Test
    fun stacksPagesIntoOneContinuousColumn() {
        val (_, state) = show(longLesson())
        assertTrue(state.contentHeight > 2 * 800)
    }

    @Test
    fun doesNotReachEndWhenContentOverflows() {
        val (_, state) = show(longLesson())
        assertFalse(state.reachedEnd)
    }

    @Test
    fun scrollingToTheLastAnchorReachesEnd() {
        val (_, state) = show(longLesson())
        scrollTo(state, LessonTarget.Anchor("alvo"))
        assertTrue(state.reachedEnd)
    }

    @Test
    fun scrollToAnchorHighlightsThenClears() {
        val (_, state) = show(longLesson())
        assertNull(state.highlighted)
        scrollTo(state, LessonTarget.Anchor("alvo"))
        assertEquals("alvo", state.highlighted)
        assertTrue(state.scrollOffset > 0)
        compose.mainClock.advanceTimeBy(2_000)
        compose.waitForIdle()
        assertNull(state.highlighted)
    }

    @Test
    fun scrollToPageMovesToThatPage() {
        val (document, state) = show(longLesson())
        scrollTo(state, LessonTarget.Page(document.pages[1].id))
        assertTrue(state.scrollOffset > 0)
        assertEquals(document.pages[1].id, state.highlighted)
    }

    @Test
    fun unknownTargetsAreIgnored() {
        val (_, state) = show(longLesson())
        scrollTo(state, LessonTarget.Anchor("nao-existe"))
        assertEquals(0, state.scrollOffset)
        assertNull(state.highlighted)
    }

    @Test
    fun scrollToBeforeFirstLayoutStillScrollsAndHighlights() {
        val state = showWithEarlyScroll(longLesson(), LessonTarget.Anchor("alvo"))
        assertEquals("alvo", state.highlighted)
        assertTrue(state.scrollOffset > 0)
        assertTrue(state.reachedEnd)
    }

    @Test
    fun scrollToPageDrawsAVisibleHighlightThatClears() {
        val (document, state) = show(longLesson())
        scrollTo(state, LessonTarget.Page(document.pages[1].id))
        val highlighted = snapshot()
        compose.mainClock.advanceTimeBy(2_000)
        compose.waitForIdle()
        assertNull(state.highlighted)
        val cleared = snapshot()
        assertFalse(highlighted.contentEquals(cleared))
    }

    @Test
    fun scrollToListItemAnchorScrollsAndHighlights() {
        val source = longLesson() + "\n\n- primeiro item\n- segundo item {#item-final}"
        val (_, state) = show(source)
        scrollTo(state, LessonTarget.Anchor("item-final"))
        assertEquals("item-final", state.highlighted)
        assertTrue(state.scrollOffset > 0)
    }
}
