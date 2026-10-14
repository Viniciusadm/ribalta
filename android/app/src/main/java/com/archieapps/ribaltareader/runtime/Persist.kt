package com.archieapps.ribaltareader.runtime

import android.content.Context
import com.archieapps.ribaltareader.ir.Document
import com.archieapps.ribaltareader.ir.IrParser
import com.archieapps.ribaltareader.ir.Scalar
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.jsonObject

class Persist(context: Context, private val source: String) {
    private val preferences = context.getSharedPreferences("ribalta.persist", Context.MODE_PRIVATE)

    private fun key(document: Document) = "$source|${document.meta.title ?: ""}"

    private fun kept(document: Document, state: Map<String, Scalar>) = state.filterKeys { it in document.persist }

    fun load(document: Document): Map<String, Scalar> {
        if (document.persist.isEmpty()) return emptyMap()
        val raw = preferences.getString(key(document), null) ?: return emptyMap()
        return runCatching { kept(document, Json.parseToJsonElement(raw).jsonObject.mapValues { (_, value) -> IrParser.scalar(value) }) }.getOrDefault(emptyMap())
    }

    fun save(document: Document, state: Map<String, Scalar>) {
        if (document.persist.isEmpty()) return
        val json = JsonObject(kept(document, state).mapValues { (_, value) -> encode(value) })
        preferences.edit().putString(key(document), json.toString()).apply()
    }

    private fun encode(value: Scalar): JsonElement = when (value) {
        is Scalar.Bool -> JsonPrimitive(value.value)
        is Scalar.Number -> JsonPrimitive(value.value)
        is Scalar.Text -> JsonPrimitive(value.value)
        is Scalar.List -> JsonArray(value.items.map(::JsonPrimitive))
        is Scalar.Records -> JsonArray(value.rows.map { row -> JsonObject(row.mapValues { (_, field) -> encode(field) }) })
    }
}
