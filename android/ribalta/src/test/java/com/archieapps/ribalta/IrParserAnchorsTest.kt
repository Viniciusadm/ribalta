package com.archieapps.ribalta

import com.archieapps.ribalta.compiler.RibaltaCompiler
import com.archieapps.ribalta.ir.BlockKind
import com.archieapps.ribalta.ir.IrParser
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class IrParserAnchorsTest {
    private val lessonOptions = """{"page":{"width":400,"height":null},"profile":"lesson"}"""

    private fun lesson(source: String) =
        IrParser.compilation(RibaltaCompiler.compileJson(source, options = lessonOptions)).document

    @Test
    fun readsBlockIdAndDocumentAnchors() {
        val document = lesson("# Titulo\n\nTexto. {#fase-clara}\n\n--- {#segunda}\n\nOutro texto. {#fase-escura}")
        val ids = document.pages.flatMap { page -> page.children.mapNotNull { it.id } }
        assertEquals(listOf("fase-clara", "fase-escura"), ids)
        assertEquals(0, document.anchors["fase-clara"])
        assertEquals(1, document.anchors["fase-escura"])
    }

    @Test
    fun readsListItemId() {
        val document = lesson("- primeiro {#item-um}\n- segundo")
        val list = document.pages[0].children[0].kind as BlockKind.BulletList
        assertEquals("item-um", list.items[0].id)
        assertNull(list.items[1].id)
    }

    @Test
    fun documentsWithoutAnchorsHaveNoneAndBlocksHaveNoId() {
        val document = IrParser.compilation(RibaltaCompiler.compileJson("# So titulo")).document
        assertEquals(emptyMap<String, Int>(), document.anchors)
        assertNull(document.pages[0].children[0].id)
    }
}
