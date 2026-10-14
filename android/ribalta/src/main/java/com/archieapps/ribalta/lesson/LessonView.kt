package com.archieapps.ribalta.lesson

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.archieapps.ribalta.ir.Document
import com.archieapps.ribalta.render.Fonts
import com.archieapps.ribalta.render.Images
import com.archieapps.ribalta.render.Interaction
import com.archieapps.ribalta.render.LocalAnchors
import com.archieapps.ribalta.render.LocalInteraction
import com.archieapps.ribalta.render.PageView
import com.archieapps.ribalta.render.RenderContext
import com.archieapps.ribalta.render.anchored
import com.archieapps.ribalta.render.pageBackground
import com.archieapps.ribalta.runtime.Navigation
import com.archieapps.ribalta.runtime.Scope

private const val PAGE_HIGHLIGHT_HEIGHT = 120

@Composable
fun LessonView(document: Document, state: LessonViewState, modifier: Modifier = Modifier) {
    val context = LocalContext.current
    val fonts = remember { Fonts(context.applicationContext) }
    val images = remember { Images(null) }
    var fontsLoaded by remember(document) { mutableIntStateOf(0) }
    LaunchedEffect(document) {
        fonts.load(document.theme.fontFamilies)
        fontsLoaded++
    }
    val pageWidth = document.pages.firstOrNull()?.width ?: 1f
    BoxWithConstraints(
        modifier
            .onGloballyPositioned { state.container = it }
            .onSizeChanged { state.viewportHeight = it.height },
    ) {
        val outer = LocalDensity.current
        val scale = if (maxWidth == Dp.Infinity) 1f else with(outer) { maxWidth.toPx() / pageWidth }
        CompositionLocalProvider(LocalAnchors provides state.anchors, LocalDensity provides Density(scale, outer.fontScale)) {
            Column(Modifier.fillMaxWidth().verticalScroll(state.scrollState)) {
                document.pages.forEachIndexed { index, page ->
                    val session = remember(document, page) { Navigation.start(document, index).copy(step = page.steps) }
                    val render = remember(document, page, fontsLoaded) {
                        RenderContext(document.theme, fonts, document.icons, images, pageBackground(document, page), false)
                    }
                    val interaction = remember(session) { Interaction(Scope(session.state, session.step), live = false) {} }
                    CompositionLocalProvider(LocalInteraction provides interaction) {
                        Column(Modifier.width(pageWidth.dp).anchored(page.id, key = LessonViewState.pageKey(page.id), highlightHeight = PAGE_HIGHLIGHT_HEIGHT.dp)) {
                            PageView(document, page, render)
                        }
                    }
                }
            }
        }
    }
}
