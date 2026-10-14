package com.archieapps.ribalta.lesson

import androidx.compose.foundation.ScrollState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.layout.LayoutCoordinates
import androidx.compose.ui.layout.positionInRoot
import com.archieapps.ribalta.render.AnchorRegistry
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlin.math.roundToInt

private const val HIGHLIGHT_MILLIS = 1_500L
private const val PAGE_KEY_PREFIX = "page:"

class LessonViewState internal constructor(private val scope: CoroutineScope) {
    internal val scrollState = ScrollState(0)
    internal val anchors = AnchorRegistry()
    internal var container by mutableStateOf<LayoutCoordinates?>(null)
    private var clearing: Job? = null

    val reachedEnd: Boolean get() = !scrollState.canScrollForward

    val highlighted: String? get() = anchors.highlighted

    val scrollOffset: Int get() = scrollState.value

    val contentHeight: Int get() = if (scrollState.maxValue == Int.MAX_VALUE) 0 else scrollState.maxValue + viewportHeight

    internal var viewportHeight by mutableStateOf(0)

    suspend fun scrollTo(target: LessonTarget) {
        val (id, key) = when (target) {
            is LessonTarget.Page -> target.id to PAGE_KEY_PREFIX + target.id
            is LessonTarget.Anchor -> target.id to target.id
        }
        val coordinates = anchors.coordinates(key) ?: return
        val origin = container?.takeIf { it.isAttached } ?: return
        val offset = scrollState.value + (coordinates.positionInRoot().y - origin.positionInRoot().y)
        highlight(id)
        scrollState.animateScrollTo(offset.roundToInt().coerceAtLeast(0))
    }

    private fun highlight(id: String) {
        clearing?.cancel()
        anchors.highlighted = id
        clearing = scope.launch {
            delay(HIGHLIGHT_MILLIS)
            if (anchors.highlighted == id) anchors.highlighted = null
        }
    }

    companion object {
        internal fun pageKey(id: String) = PAGE_KEY_PREFIX + id
    }
}

@Composable
fun rememberLessonViewState(): LessonViewState {
    val scope = rememberCoroutineScope()
    return remember { LessonViewState(scope) }
}
