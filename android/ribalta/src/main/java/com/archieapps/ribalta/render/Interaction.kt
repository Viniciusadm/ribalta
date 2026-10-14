package com.archieapps.ribalta.render

import androidx.compose.runtime.compositionLocalOf
import com.archieapps.ribalta.ir.Action
import com.archieapps.ribalta.ir.Expr
import com.archieapps.ribalta.ir.Style
import com.archieapps.ribalta.ir.Variant
import com.archieapps.ribalta.runtime.Evaluator
import com.archieapps.ribalta.runtime.Scope

class Interaction(
    val scope: Scope,
    val live: Boolean = true,
    val shortcut: (String) -> Boolean = { false },
    val run: (List<Action>) -> Unit,
) {
    val step get() = scope.step

    fun holds(expr: Expr?) = Evaluator.holds(expr, scope)

    fun withVariants(style: Style, variants: List<Variant>, hovered: Boolean = false): Style {
        val variantScope = if (hovered) scope.copy(hovered = true) else scope
        return variants.fold(style) { merged, variant -> if (Evaluator.holds(variant.whenExpr, variantScope)) variant.style over merged else merged }
    }
}

val LocalInteraction = compositionLocalOf { Interaction(Scope(emptyMap(), Int.MAX_VALUE), live = false) {} }
