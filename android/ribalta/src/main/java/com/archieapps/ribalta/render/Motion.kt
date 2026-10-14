package com.archieapps.ribalta.render

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.GraphicsLayerScope
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.unit.dp
import com.archieapps.ribalta.ir.Animation
import com.archieapps.ribalta.ir.EnterKind
import com.archieapps.ribalta.ir.Length
import com.archieapps.ribalta.ir.Motion
import com.archieapps.ribalta.ir.Pose
import com.archieapps.ribalta.ir.Timing

fun easing(timing: Timing) = timing.easing.let { CubicBezierEasing(it.getOrElse(0) { 0f }, it.getOrElse(1) { 0f }, it.getOrElse(2) { 1f }, it.getOrElse(3) { 1f }) }

private fun GraphicsLayerScope.offset(length: Length?, reference: Float): Float = when (length) {
    is Length.Px -> length.value.dp.toPx()
    is Length.Pct -> reference * length.value / 100f
    else -> 0f
}

fun GraphicsLayerScope.applyPose(pose: Pose, progress: Float) {
    val rest = 1f - progress
    alpha = pose.opacity + (1f - pose.opacity) * progress
    translationX = offset(pose.dx, size.width) * rest
    translationY = offset(pose.dy, size.height) * rest
    val scale = (pose.scale ?: 1f) + (1f - (pose.scale ?: 1f)) * progress
    scaleX = scale
    scaleY = scale
}

@Composable
fun Modifier.reveal(step: Int?, enter: EnterKind?, motion: Motion, current: Int): Modifier {
    if (step == null) return this
    val shown = current >= step
    val kind = enter ?: EnterKind.Up
    if (kind == EnterKind.None) return graphicsLayer { alpha = if (shown) 1f else 0f }
    val pose = motion.enter[kind]?.from ?: return graphicsLayer { alpha = if (shown) 1f else 0f }
    val progress by animateFloatAsState(if (shown) 1f else 0f, tween(motion.reveal.duration, easing = easing(motion.reveal)), label = "reveal")
    return graphicsLayer { applyPose(pose, progress) }
}

@Composable
fun Modifier.entrance(animation: Animation?): Modifier {
    if (animation == null) return this
    val progress = remember { Animatable(0f) }
    LaunchedEffect(Unit) { progress.animateTo(1f, tween(animation.timing.duration, easing = easing(animation.timing))) }
    return graphicsLayer { applyPose(animation.from, progress.value) }
}

