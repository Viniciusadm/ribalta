package com.archieapps.ribalta.render

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.LayoutCoordinates
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

class AnchorRegistry {
    private val positions = mutableMapOf<String, LayoutCoordinates>()
    var highlightedKey by mutableStateOf<String?>(null)
    var highlightedId by mutableStateOf<String?>(null)
    var version by mutableIntStateOf(0)
        private set

    fun register(key: String, coordinates: LayoutCoordinates) {
        val known = positions.put(key, coordinates)
        if (known == null) version++
    }

    fun unregister(key: String, coordinates: LayoutCoordinates?) {
        if (coordinates != null && positions[key] === coordinates) {
            positions.remove(key)
            version++
        }
    }

    fun coordinates(key: String): LayoutCoordinates? = positions[key]?.takeIf { it.isAttached }
}

val LocalAnchors = compositionLocalOf<AnchorRegistry?> { null }

private val HighlightColor = Color(0x66FFC107)

private class Registration {
    var coordinates: LayoutCoordinates? = null
}

@Composable
fun Modifier.anchored(id: String?, key: String? = id, highlightHeight: Dp? = null): Modifier {
    val registry = LocalAnchors.current
    if (id == null || key == null || registry == null) return this
    val registration = remember(registry, key) { Registration() }
    DisposableEffect(registry, key) { onDispose { registry.unregister(key, registration.coordinates) } }
    return this
        .onGloballyPositioned {
            registration.coordinates = it
            registry.register(key, it)
        }
        .drawWithContent {
            drawContent()
            if (registry.highlightedKey == key) {
                val height = highlightHeight?.toPx()?.coerceAtMost(size.height) ?: size.height
                drawRoundRect(HighlightColor, size = Size(size.width, height), cornerRadius = CornerRadius(8.dp.toPx()))
            }
        }
}
