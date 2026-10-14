package com.archieapps.ribalta.render

import android.content.Context
import androidx.compose.ui.text.ExperimentalTextApi
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontVariation
import androidx.compose.ui.text.font.FontWeight
import com.archieapps.ribalta.ir.FontFamilySpec
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.File
import java.net.HttpURLConnection
import java.net.URL
import java.net.URLEncoder

private val weights = (100..900 step 100).toList()

private val variableFonts = mapOf(
    "DM Sans" to "DMSans-Variable.ttf",
    "Fraunces" to "Fraunces-Variable.ttf",
    "Inter" to "Inter-Variable.ttf",
    "Inter Tight" to "InterTight-Variable.ttf",
    "JetBrains Mono" to "JetBrainsMono-Variable.ttf",
    "Nunito" to "Nunito-Variable.ttf",
    "Playfair Display" to "PlayfairDisplay-Variable.ttf",
    "Source Sans 3" to "SourceSans3-Variable.ttf",
    "Space Grotesk" to "SpaceGrotesk-Variable.ttf",
)

private val pinnedAxes = mapOf(
    "DM Sans" to listOf("opsz" to 14f),
    "Inter" to listOf("opsz" to 14f),
    "Fraunces" to listOf("opsz" to 14f, "SOFT" to 0f, "WONK" to 0f),
)

private val barlowFiles = mapOf(
    100 to "Thin", 200 to "ExtraLight", 300 to "Light", 400 to "Regular", 500 to "Medium",
    600 to "SemiBold", 700 to "Bold", 800 to "ExtraBold", 900 to "Black",
)

class Fonts(private val context: Context) {
    private val families = mutableMapOf<String, FontFamily>()

    @OptIn(ExperimentalTextApi::class)
    private fun bundled(name: String): FontFamily? {
        val assets = context.assets
        variableFonts[name]?.let { file ->
            val pinned = pinnedAxes[name].orEmpty().map { (axis, value) -> FontVariation.Setting(axis, value) }
            return FontFamily(weights.map { weight ->
                Font("fonts/$file", assets, FontWeight(weight), variationSettings = FontVariation.Settings(*(pinned + FontVariation.weight(weight)).toTypedArray()))
            })
        }
        return when (name) {
            "Barlow" -> FontFamily(barlowFiles.map { (weight, file) -> Font("fonts/Barlow-$file.ttf", assets, FontWeight(weight)) })
            "Archivo Black" -> FontFamily(Font("fonts/ArchivoBlack-Regular.ttf", assets, FontWeight.Normal))
            else -> null
        }
    }

    fun family(name: String?): FontFamily {
        if (name == null) return FontFamily.SansSerif
        return families.getOrPut(name) { bundled(name) ?: FontFamily.SansSerif }
    }

    suspend fun load(specs: List<FontFamilySpec>) = withContext(Dispatchers.IO) {
        for (spec in specs) {
            if (spec.family in families || bundled(spec.family) != null) continue
            runCatching { downloaded(spec) }.getOrNull()?.let { families[spec.family] = it }
        }
    }

    private fun downloaded(spec: FontFamilySpec): FontFamily? {
        val query = URLEncoder.encode(spec.family, "UTF-8").replace("%20", "+") + if (spec.weights.isEmpty()) "" else ":wght@" + spec.weights.joinToString(";")
        val css = fetch("https://fonts.googleapis.com/css2?family=$query").toString(Charsets.UTF_8)
        val faces = Regex("""font-weight:\s*(\d+);[^}]*?url\((https://[^)]+\.ttf)\)""").findAll(css).map { it.groupValues[1].toInt() to it.groupValues[2] }.toList()
        if (faces.isEmpty()) return null
        val folder = File(context.cacheDir, "fonts").apply { mkdirs() }
        return FontFamily(faces.map { (weight, url) ->
            val file = File(folder, url.substringAfterLast('/'))
            if (!file.exists()) file.writeBytes(fetch(url))
            Font(file, FontWeight(weight))
        })
    }

    private fun fetch(address: String): ByteArray {
        val connection = URL(address).openConnection() as HttpURLConnection
        connection.setRequestProperty("User-Agent", "Ribalta")
        connection.connectTimeout = 10_000
        connection.readTimeout = 10_000
        return connection.inputStream.use { it.readBytes() }
    }
}
