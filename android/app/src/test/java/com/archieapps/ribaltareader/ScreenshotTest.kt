package com.archieapps.ribaltareader

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
import com.archieapps.ribaltareader.compiler.RibaltaCompiler
import com.archieapps.ribaltareader.render.Fonts
import com.archieapps.ribaltareader.render.Images
import com.archieapps.ribaltareader.render.Interaction
import com.archieapps.ribaltareader.render.LocalInteraction
import com.archieapps.ribaltareader.render.RenderContext
import com.archieapps.ribaltareader.render.PageView
import com.archieapps.ribaltareader.render.slideBackground
import com.archieapps.ribaltareader.runtime.Navigation
import com.archieapps.ribaltareader.runtime.Scope
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
    fun rendersDecksToPng() {
        val decks = System.getProperty("ribalta.decks")?.split(File.pathSeparator)?.filter { it.isNotBlank() }
        assumeTrue("set RIBALTA_DECKS to render screenshots", !decks.isNullOrEmpty())
        val output = File(System.getProperty("ribalta.screenshots")!!).apply { mkdirs() }
        val fonts = Fonts(ApplicationProvider.getApplicationContext())
        var deckIndex by mutableIntStateOf(0)
        var slideIndex by mutableIntStateOf(0)
        val compilations = decks!!.map { RibaltaCompiler.compile(File(it).readText()) }
        compilations.forEach { runBlocking { fonts.load(it.document.theme.fontFamilies) } }
        compose.setContent {
            val document = compilations[deckIndex].document
            val slide = document.pages[slideIndex]
            val session = Navigation.start(document, slideIndex).copy(step = slide.steps)
            val render = RenderContext(document.theme, fonts, document.icons, Images(null), slideBackground(document, slide), false)
            CompositionLocalProvider(LocalDensity provides Density(1f), LocalInteraction provides Interaction(Scope(session.state, session.step), live = false) {}) {
                Box { PageView(document, slide, render) }
            }
        }
        compilations.forEachIndexed { d, compilation ->
            val name = File(decks[d]).nameWithoutExtension
            compilation.document.pages.indices.forEach { s ->
                compose.runOnIdle {
                    deckIndex = d
                    slideIndex = s
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
