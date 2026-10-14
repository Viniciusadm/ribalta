package com.archieapps.ribaltareader.runtime

import com.archieapps.ribaltareader.ir.Builtin
import com.archieapps.ribaltareader.ir.Expr
import com.archieapps.ribaltareader.ir.Scalar
import java.math.BigDecimal
import kotlin.math.abs
import kotlin.math.ceil
import kotlin.math.floor
import kotlin.math.max
import kotlin.math.min
import kotlin.math.pow
import kotlin.math.sign
import kotlin.math.truncate

data class Scope(
    val state: Map<String, Scalar>,
    val step: Int,
    val elapsed: Int = 0,
    val hovered: Boolean = false,
    val draw: (() -> Double)? = null,
    val items: Map<String, Scalar> = emptyMap(),
    val rows: Map<String, Map<String, Scalar>> = emptyMap(),
)

data class Draw(val value: Double, val seed: Int)

object Evaluator {
    fun nextRandom(seed: Int): Draw {
        val next = seed + 0x6D2B79F5
        var mixed = (next xor (next ushr 15)) * (next or 1)
        mixed = mixed xor (mixed + (mixed xor (mixed ushr 7)) * (mixed or 61))
        return Draw((mixed xor (mixed ushr 14)).toUInt().toDouble() / 4294967296.0, next)
    }

    fun format(value: Scalar): String = when (value) {
        is Scalar.Bool -> value.value.toString()
        is Scalar.Text -> value.value
        is Scalar.Number -> formatNumber(value.value)
        is Scalar.List -> value.items.joinToString(", ")
        is Scalar.Records -> ""
    }

    private fun formatNumber(value: Double): String {
        if (value == 0.0) return "0"
        if (value.isNaN()) return "NaN"
        if (value.isInfinite()) return if (value > 0) "Infinity" else "-Infinity"
        if (value == truncate(value) && abs(value) < 1e15) return BigDecimal(value).toPlainString()
        val magnitude = abs(value)
        if (magnitude >= 1e21 || magnitude < 1e-6) {
            val (mantissa, exponent) = value.toString().split('E').let { it[0].removeSuffix(".0") to it.getOrElse(1) { "0" }.toInt() }
            return "${mantissa}e${if (exponent >= 0) "+" else ""}$exponent"
        }
        return BigDecimal(value.toString()).stripTrailingZeros().toPlainString()
    }

    fun holds(expr: Expr?, scope: Scope) = expr == null || evaluate(expr, scope) == Scalar.Bool(true)

    private fun number(value: Scalar): Double = when (value) {
        is Scalar.Number -> value.value
        is Scalar.Bool -> if (value.value) 1.0 else 0.0
        is Scalar.Text -> value.value.trim().let { if (it.isEmpty()) 0.0 else it.toDoubleOrNull()?.takeUnless { parsed -> parsed.isNaN() } ?: 0.0 }
        is Scalar.List -> 0.0
        is Scalar.Records -> 0.0
    }

    private fun length(value: Scalar) = when (value) {
        is Scalar.List -> value.items.size
        is Scalar.Records -> value.rows.size
        is Scalar.Text -> value.value.codePointCount(0, value.value.length)
        else -> 0
    }

    private fun truthy(value: Scalar) = when (value) {
        is Scalar.Bool -> value.value
        is Scalar.Number -> value.value != 0.0 && !value.value.isNaN()
        is Scalar.Text -> value.value.isNotEmpty()
        is Scalar.List -> true
        is Scalar.Records -> true
    }

    private fun roundTo(value: Double, digits: Double): Double {
        val factor = 10.0.pow(truncate(min(max(digits, 0.0), 10.0)))
        val scaled = value * factor
        return sign(scaled) * floor(abs(scaled) + 0.5) / factor
    }

    private fun compare(left: Scalar, right: Scalar): Int =
        if (left is Scalar.Text && right is Scalar.Text) left.value.compareTo(right.value) else number(left).compareTo(number(right))

    fun evaluate(expr: Expr, scope: Scope): Scalar = when (expr) {
        is Expr.Lit -> expr.value
        is Expr.State -> scope.state[expr.name] ?: Scalar.Bool(false)
        is Expr.Item -> (if (expr.field == null) scope.items[expr.name] else scope.rows[expr.name]?.get(expr.field)) ?: Scalar.Bool(false)
        Expr.Step -> Scalar.Number(scope.step.toDouble())
        Expr.Elapsed -> Scalar.Number(scope.elapsed.toDouble())
        Expr.Hover -> Scalar.Bool(scope.hovered)
        is Expr.Not -> Scalar.Bool(!truthy(evaluate(expr.arg, scope)))
        is Expr.Neg -> Scalar.Number(-number(evaluate(expr.arg, scope)))
        is Expr.Cond -> if (evaluate(expr.test, scope) == Scalar.Bool(true)) evaluate(expr.then, scope) else evaluate(expr.otherwise, scope)
        is Expr.Call -> call(expr, scope)
        is Expr.Binary -> binary(expr, scope)
    }

    private fun call(expr: Expr.Call, scope: Scope): Scalar {
        when (expr.name) {
            Builtin.Len -> return Scalar.Number(length(expr.args.firstOrNull()?.let { evaluate(it, scope) } ?: Scalar.Text("")).toDouble())
            Builtin.Pick -> {
                val items = expr.args.flatMap { arg -> evaluate(arg, scope).let { if (it is Scalar.List) it.items.map(Scalar::Text) else listOf(it) } }
                val draw = scope.draw
                if (items.isEmpty() || draw == null) return Scalar.Text("")
                return items[floor(draw() * items.size).toInt().coerceAtMost(items.size - 1)]
            }
            Builtin.Random -> {
                val (a, b) = expr.args.map { number(evaluate(it, scope)) }.let { it.getOrElse(0) { 0.0 } to it.getOrElse(1) { 0.0 } }
                val low = ceil(min(a, b))
                val high = max(low, floor(max(a, b)))
                val draw = scope.draw ?: return Scalar.Number(low)
                return Scalar.Number(low + floor(draw() * (high - low + 1)))
            }
            else -> Unit
        }
        val args = expr.args.map { number(evaluate(it, scope)) }
        return Scalar.Number(
            when (expr.name) {
                Builtin.Round -> roundTo(args.getOrElse(0) { 0.0 }, args.getOrElse(1) { 0.0 })
                Builtin.Min -> args.minOrNull() ?: Double.POSITIVE_INFINITY
                else -> args.maxOrNull() ?: Double.NEGATIVE_INFINITY
            },
        )
    }

    private fun binary(expr: Expr.Binary, scope: Scope): Scalar {
        if (expr.operator == "and") return Scalar.Bool(holds(expr.left, scope) && holds(expr.right, scope))
        if (expr.operator == "or") return Scalar.Bool(holds(expr.left, scope) || holds(expr.right, scope))
        val left = evaluate(expr.left, scope)
        val right = evaluate(expr.right, scope)
        return when (expr.operator) {
            "in" -> Scalar.Bool(right is Scalar.List && left is Scalar.Text && left.value in right.items)
            "==" -> Scalar.Bool(left == right)
            "!=" -> Scalar.Bool(left != right)
            "<" -> Scalar.Bool(compare(left, right) < 0)
            "<=" -> Scalar.Bool(compare(left, right) <= 0)
            ">" -> Scalar.Bool(compare(left, right) > 0)
            ">=" -> Scalar.Bool(compare(left, right) >= 0)
            "+" -> if (left is Scalar.Text || right is Scalar.Text) Scalar.Text(format(left) + format(right)) else Scalar.Number(number(left) + number(right))
            "-" -> Scalar.Number(number(left) - number(right))
            "*" -> Scalar.Number(number(left) * number(right))
            "/" -> Scalar.Number(if (number(right) == 0.0) 0.0 else number(left) / number(right))
            "%" -> Scalar.Number(if (number(right) == 0.0) 0.0 else number(left) % number(right))
            else -> Scalar.Bool(false)
        }
    }
}
