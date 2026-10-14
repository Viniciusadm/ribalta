package com.archieapps.ribalta

import com.archieapps.ribalta.compiler.RibaltaCompiler
import com.archieapps.ribalta.ir.Action
import com.archieapps.ribalta.ir.BlockKind
import com.archieapps.ribalta.ir.Expr
import com.archieapps.ribalta.ir.Inline
import com.archieapps.ribalta.ir.Scalar
import com.archieapps.ribalta.runtime.Evaluator
import com.archieapps.ribalta.runtime.Navigation
import com.archieapps.ribalta.runtime.Scope
import com.archieapps.ribalta.runtime.Shortcuts
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.File

class CompilerTest {
    @Test
    fun compilesThroughTheNativeLibraryAndKeepsUnicode() {
        val compilation = RibaltaCompiler.compile("# Ação 🚀\n\n:: icon rocket")
        val heading = compilation.document.pages[0].children[0].kind as BlockKind.Heading
        assertEquals(Inline.Text("Ação 🚀"), heading.content[0])
        assertTrue(compilation.document.icons.containsKey("rocket"))
    }

    @Test
    fun readsRuntimeRepetitionOverExternRecords() {
        val source = "@data exemplo\n| nome | pontos |\n| --- | --- |\n| Ana | 300 |\n@end\n@extern placar = \$exemplo\n\n# Placar\n\n::: for jogador, i in \$placar\n\$i. \$jogador.nome\n:::"
        val document = RibaltaCompiler.compile(source).document
        val placar = document.state["placar"] as Scalar.Records
        assertEquals(Scalar.Number(300.0), placar.rows[0]["pontos"])
        val each = document.pages[0].children[1].kind as BlockKind.Each
        assertEquals("jogador", each.item)
        assertEquals("i", each.index)
        val scope = Scope(document.state, 0, items = mapOf("i" to Scalar.Number(1.0)), rows = mapOf("jogador" to placar.rows[0]))
        assertEquals(Scalar.Text("Ana"), Evaluator.evaluate(Expr.Item("jogador", "nome"), scope))
        assertEquals(Scalar.Number(1.0), Evaluator.evaluate(Expr.Item("i", null), scope))
    }

    @Test
    fun everyDocumentationExampleParses() {
        val pages = File(System.getProperty("ribalta.docs")!!).walkTopDown().filter { it.isFile && it.extension == "md" }.toList()
        var examples = 0
        var compiled = 0
        for (page in pages) {
            for (source in ribaltaFences(page.readText())) {
                compiled += RibaltaCompiler.compile(source).document.pages.size
                examples += 1
            }
        }
        assertTrue("only $examples examples", examples > 300)
        assertTrue("only $compiled pages", compiled >= examples)
    }

    private fun ribaltaFences(markdown: String): List<String> {
        val fences = mutableListOf<String>()
        var open = 0
        var lines: MutableList<String>? = null
        for (line in markdown.lines()) {
            val trimmed = line.trimStart()
            val ticks = trimmed.takeWhile { it == '`' }.length
            val current = lines
            if (current != null) {
                if (ticks >= open && trimmed.drop(ticks).isBlank()) {
                    fences += current.joinToString("\n")
                    lines = null
                } else {
                    current += line
                }
            } else if (ticks >= 3) {
                val info = trimmed.drop(ticks).trim()
                if (info == "ribalta" || info.startsWith("ribalta ")) {
                    open = ticks
                    lines = mutableListOf()
                }
            }
        }
        return fences
    }

    @Test
    fun readsInteractiveFeaturesAddedAfterTheFirstReader() {
        val source = """
            @mode game
            @state name = ""
            @state endings = []
            @state die = 0
            @persist endings

            --- {on-enter="add ${'$'}endings start; ${'$'}die = random(1, 6)" timeout=5 on-timeout="go #end"}
            # Hello
            :: input {bind=${'$'}name placeholder="Name" on-submit="go #end"}
            :: sound https://example.com/rain.mp3 {name=rain loop volume=40%}
            [Play]{click="play rain" key=p}

            --- {#end}
            # ${'$'}elapsed
        """.trimIndent()
        val compilation = RibaltaCompiler.compile(source, english = true)
        assertTrue(compilation.diagnostics.toString(), compilation.diagnostics.none { it.severity == "error" })
        val document = compilation.document
        assertTrue(document.meta.game)
        assertTrue(document.meta.random)
        assertEquals(listOf("endings"), document.persist)
        assertEquals(Scalar.List(emptyList()), document.state["endings"])
        val first = document.pages[0]
        assertTrue(first.locked && first.clock)
        assertEquals(5.0, first.timeout)
        assertEquals(listOf(Action.Go("end")), first.onTimeout)
        assertTrue(first.onEnter[0] is Action.Add)
        val input = first.children.map { it.kind }.filterIsInstance<BlockKind.Input>().single()
        assertEquals("name", input.name)
        assertEquals(listOf(Action.Go("end")), input.onSubmit)
        val sound = first.children.map { it.kind }.filterIsInstance<BlockKind.Sound>().single()
        assertEquals("rain", sound.name)
        assertTrue(sound.loop)
        assertTrue(first.children.none { it.kind == BlockKind.Divider })

        val session = Navigation.start(document, seed = 7)
        assertEquals(Scalar.List(listOf("start")), session.state["endings"])
        assertTrue((session.state["die"] as Scalar.Number).value in 1.0..6.0)
        assertEquals("p", Shortcuts.forPage(document, session).keys.single())
        assertEquals(0, Navigation.advance(document, session, 1).index)
        val timedOut = (1..5).fold(session) { current, _ -> Navigation.tick(document, current) }
        assertEquals(1, timedOut.index)
    }
}
