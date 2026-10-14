package com.archieapps.ribalta

import com.archieapps.ribalta.ir.Document
import com.archieapps.ribalta.ir.IrParser
import com.archieapps.ribalta.ir.Scalar
import com.archieapps.ribalta.runtime.Evaluator
import com.archieapps.ribalta.runtime.Navigation
import com.archieapps.ribalta.runtime.Scope
import com.archieapps.ribalta.runtime.Session
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.int
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.long
import org.junit.Assert.assertEquals
import org.junit.Test

class ConformanceTest {
    private fun fixture(name: String): JsonObject =
        Json.parseToJsonElement(javaClass.classLoader!!.getResource(name)!!.readText()).jsonObject

    @Test
    fun expressionsEvaluateAndPrintLikeTheContract() {
        for (case in fixture("expressions.json").getValue("cases").jsonArray.map { it.jsonObject }) {
            val name = case.getValue("name").jsonPrimitive.content
            val state = IrParser.stateMap(case["state"]?.jsonObject)
            val step = case["step"]?.jsonPrimitive?.int ?: 0
            val elapsed = case["elapsed"]?.jsonPrimitive?.int ?: 0
            var seed = seed(case["seed"])
            val draw = {
                val result = Evaluator.nextRandom(seed)
                seed = result.seed
                result.value
            }
            val value = Evaluator.evaluate(IrParser.expr(case.getValue("expr").jsonObject), Scope(state, step, elapsed, draw = draw))
            assertEquals(name, normalize(IrParser.scalar(case.getValue("value"))), normalize(value))
            assertEquals(name, case.getValue("printed").jsonPrimitive.content, Evaluator.format(value))
        }
    }

    @Test
    fun navigationMovesLikeTheContract() {
        val fixture = fixture("navigation.json")
        val documents = fixture.getValue("documents").jsonObject
        for (case in fixture.getValue("cases").jsonArray.map { it.jsonObject }) {
            val name = case.getValue("name").jsonPrimitive.content
            val document = minimalDocument(documents.getValue(case.getValue("document").jsonPrimitive.content).jsonObject)
            val start = case.getValue("start").jsonObject
            var session = Navigation.start(document, Navigation.indexOf(document, start.getValue("page").jsonPrimitive.content), start["step"]?.jsonPrimitive?.int ?: 0, seed = seed(start["seed"]))
            val expectations = case.getValue("expect").jsonArray.map { it.jsonObject }
            case.getValue("ops").jsonArray.map { it.jsonObject }.forEachIndexed { index, operation ->
                session = apply(document, session, operation)
                val expected = expectations[index]
                val label = "$name, after operation ${index + 1}"
                assertEquals(label, expected.getValue("page").jsonPrimitive.content, document.pages[session.index].id)
                assertEquals(label, expected.getValue("step").jsonPrimitive.int, session.step)
                expected["elapsed"]?.jsonPrimitive?.int?.let { assertEquals(label, it, session.elapsed) }
                expected["state"]?.jsonObject?.let { assertEquals(label, IrParser.stateMap(it).mapValues { (_, value) -> normalize(value) }, session.state.mapValues { (_, value) -> normalize(value) }) }
            }
        }
    }

    private fun seed(json: JsonElement?) = json?.jsonPrimitive?.long?.toInt() ?: 0

    private fun normalize(value: Scalar): Scalar = if (value is Scalar.Number && value.value == 0.0) Scalar.Number(0.0) else value

    private fun apply(document: Document, session: Session, operation: JsonObject): Session = when (operation.getValue("op").jsonPrimitive.content) {
        "next" -> Navigation.advance(document, session, 1)
        "prev" -> Navigation.advance(document, session, -1)
        "back" -> Navigation.back(document, session)
        "tick" -> Navigation.tick(document, session)
        "go" -> Navigation.goTo(document, session, operation.getValue("target").jsonPrimitive.content)
        else -> Navigation.run(document, session, IrParser.actions(operation.getValue("actions").jsonArray))
    }

    private fun minimalDocument(json: JsonObject): Document {
        val full = """{"version":1,"meta":{"title":null,"author":null,"page":{"width":1920,"height":1080}},"theme":${templateTheme()},"pages":[]}"""
        val base = IrParser.document(Json.parseToJsonElement(full).jsonObject)
        val pages = json.getValue("pages").jsonArray.map { IrParser.page(it.jsonObject) }
        return base.copy(state = IrParser.stateMap(json["state"]?.jsonObject), pages = pages)
    }

    private fun templateTheme(): String {
        val compiled = com.archieapps.ribalta.compiler.RibaltaCompiler.compileJson("# x")
        return Json.parseToJsonElement(compiled).jsonObject.getValue("document").jsonObject.getValue("theme").toString()
    }
}
