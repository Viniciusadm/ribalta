package com.archieapps.ribalta

import com.archieapps.ribalta.compiler.RibaltaCompiler
import com.archieapps.ribalta.ir.IrParser
import com.archieapps.ribalta.ir.Style
import com.archieapps.ribalta.render.TextEnv
import com.archieapps.ribalta.render.searchShrink
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ShrinkTest {
    private fun document(source: String) = IrParser.compilation(RibaltaCompiler.compileJson(source)).document

    @Test
    fun readsTheMinimumSizeAndMarksThePage() {
        val document = document("# Grande {shrink=xl}\n---\n# Normal")
        assertEquals(52f, document.pages[0].children[0].style.shrink)
        assertTrue(document.pages[0].shrinks)
        assertNull(document.pages[1].children[0].style.shrink)
        assertFalse(document.pages[1].shrinks)
    }

    @Test
    fun keepsTheFullSizeWhenNothingShrinks() {
        val env = TextEnv.Initial.with(Style(size = 150f, shrink = 64f))
        assertEquals(150f, env.size)
    }

    @Test
    fun scalesDownToTheMinimum() {
        val env = TextEnv.Initial.copy(shrink = 0.5f)
        assertEquals(75f, env.with(Style(size = 150f, shrink = 64f)).size)
        assertEquals(64f, env.copy(shrink = 0.1f).with(Style(size = 150f, shrink = 64f)).size)
        assertEquals(40f, env.copy(shrink = 0f).with(Style(size = 40f, shrink = 64f)).size)
        assertEquals(150f, env.with(Style(size = 150f)).size)
    }

    @Test
    fun searchFindsTheLargestFactorThatFits() {
        assertEquals(1f, searchShrink({ true }))
        assertEquals(0f, searchShrink({ false }))
        val factor = searchShrink({ it * 1000 <= 600 })
        assertTrue(factor <= 0.6f && factor > 0.59f)
    }
}
