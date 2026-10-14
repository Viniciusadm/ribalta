package com.archieapps.ribalta

import android.graphics.Bitmap
import androidx.compose.foundation.layout.Box
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalDensity
import androidx.activity.ComponentActivity
import androidx.compose.ui.test.junit4.createAndroidComposeRule
import androidx.compose.ui.unit.Density
import androidx.test.core.app.ApplicationProvider
import com.archieapps.ribalta.compiler.RibaltaCompiler
import com.archieapps.ribalta.render.Fonts
import com.archieapps.ribalta.render.Images
import com.archieapps.ribalta.render.Interaction
import com.archieapps.ribalta.render.LocalInteraction
import com.archieapps.ribalta.render.RenderContext
import com.archieapps.ribalta.render.PageView
import com.archieapps.ribalta.render.pageBackground
import com.archieapps.ribalta.runtime.Navigation
import com.archieapps.ribalta.runtime.Scope
import kotlinx.coroutines.runBlocking
import org.junit.Assume.assumeTrue
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode
import java.io.File

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [35], qualifiers = "w1920dp-h1080dp-mdpi")
class ScreenshotTest {
    @get:Rule
    val compose = createAndroidComposeRule<ComponentActivity>()

    @Test
    fun rendersDocumentsToPng() {
        val documents = System.getProperty("ribalta.documents")?.split(File.pathSeparator)?.filter { it.isNotBlank() }
        assumeTrue("set RIBALTA_DOCUMENTS to render screenshots", !documents.isNullOrEmpty())
        val output = File(System.getProperty("ribalta.screenshots")!!).apply { mkdirs() }
        val fonts = Fonts(ApplicationProvider.getApplicationContext())
        var documentIndex by mutableIntStateOf(0)
        var pageIndex by mutableIntStateOf(0)
        val compilations = documents!!.map { RibaltaCompiler.compile(File(it).readText()) }
        compilations.forEach { runBlocking { fonts.load(it.document.theme.fontFamilies) } }
        compose.setContent {
            val document = compilations[documentIndex].document
            val page = document.pages[pageIndex]
            val session = Navigation.start(document, pageIndex).copy(step = page.steps)
            val render = RenderContext(document.theme, fonts, document.icons, Images(null), pageBackground(document, page), false)
            CompositionLocalProvider(LocalDensity provides Density(1f), LocalInteraction provides Interaction(Scope(session.state, session.step), live = false) {}) {
                Box { PageView(document, page, render) }
            }
        }
        compilations.forEachIndexed { d, compilation ->
            val name = File(documents[d]).nameWithoutExtension
            compilation.document.pages.indices.forEach { s ->
                compose.runOnIdle {
                    documentIndex = d
                    pageIndex = s
                }
                compose.mainClock.advanceTimeBy(2_000)
                compose.waitForIdle()
                val bitmap = Bitmap.createBitmap(1920, 1080, Bitmap.Config.ARGB_8888)
                compose.runOnIdle { compose.activity.window.decorView.draw(android.graphics.Canvas(bitmap)) }
                File(output, "$name-${s + 1}.png").outputStream().use { bitmap.compress(Bitmap.CompressFormat.PNG, 100, it) }
            }
        }
    }
}
