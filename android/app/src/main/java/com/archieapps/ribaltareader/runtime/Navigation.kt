package com.archieapps.ribaltareader.runtime

import com.archieapps.ribaltareader.ir.Action
import com.archieapps.ribaltareader.ir.Document
import com.archieapps.ribaltareader.ir.Scalar
import java.security.SecureRandom

data class HistoryEntry(val index: Int, val step: Int, val locals: Map<String, Scalar>)

enum class CueType { Play, Stop }

data class Cue(val id: Int, val type: CueType, val name: String)

data class Session(
    val index: Int,
    val step: Int,
    val state: Map<String, Scalar>,
    val history: List<HistoryEntry>,
    val rng: Int = 0,
    val elapsed: Int = 0,
    val cues: List<Cue> = emptyList(),
)

object Navigation {
    private const val KEPT_CUES = 32

    fun freshSeed() = SecureRandom().nextInt()

    private fun locals(document: Document, index: Int, state: Map<String, Scalar>): Map<String, Scalar> {
        val own = document.pages.getOrNull(index)?.state ?: emptyMap()
        return own.mapValues { (name, initial) -> state[name] ?: initial }
    }

    private fun withoutLocals(document: Document, index: Int, state: Map<String, Scalar>): Map<String, Scalar> {
        val own = document.pages.getOrNull(index)?.state ?: emptyMap()
        return state.filterKeys { it !in own }
    }

    private fun place(document: Document, session: Session, index: Int, step: Int, restored: Map<String, Scalar>? = null): Session {
        val target = index.coerceIn(0, maxOf(document.pages.size - 1, 0))
        val steps = document.pages.getOrNull(target)?.steps ?: 0
        val globals = withoutLocals(document, session.index, session.state)
        val state = globals + (document.pages.getOrNull(target)?.state ?: emptyMap()) + (restored ?: emptyMap())
        return session.copy(index = target, step = step.coerceIn(0, steps), state = state, elapsed = 0)
    }

    private fun changeState(session: Session, action: Action): Session {
        var rng = session.rng
        val draw = {
            val result = Evaluator.nextRandom(rng)
            rng = result.seed
            result.value
        }
        val scope = Scope(session.state, session.step, session.elapsed, draw = draw)
        val changed = applyChange(session, action, scope)
        return if (rng == session.rng) changed else changed.copy(rng = rng)
    }

    private fun applyChange(session: Session, action: Action, scope: Scope): Session {
        fun list(name: String) = (session.state[name] as? Scalar.List)?.items ?: emptyList()
        fun write(name: String, value: Scalar) = session.copy(state = session.state + (name to value))
        fun cue(type: CueType, name: String): Session {
            val id = (session.cues.lastOrNull()?.id ?: 0) + 1
            return session.copy(cues = (session.cues + Cue(id, type, name)).takeLast(KEPT_CUES))
        }
        return when (action) {
            is Action.Set -> write(action.name, Evaluator.evaluate(action.value, scope))
            is Action.Toggle -> write(action.name, Scalar.Bool(session.state[action.name] != Scalar.Bool(true)))
            is Action.Add -> {
                val item = Evaluator.format(Evaluator.evaluate(action.value, scope))
                val items = list(action.name)
                if (item in items) session else write(action.name, Scalar.List(items + item))
            }
            is Action.Remove -> {
                val item = Evaluator.format(Evaluator.evaluate(action.value, scope))
                write(action.name, Scalar.List(list(action.name).filter { it != item }))
            }
            is Action.Clear -> write(action.name, Scalar.List(emptyList()))
            is Action.Play -> cue(CueType.Play, action.name)
            is Action.Stop -> cue(CueType.Stop, action.name)
            else -> session
        }
    }

    private fun arrive(document: Document, session: Session): Session =
        (document.pages.getOrNull(session.index)?.onEnter ?: emptyList()).fold(session, ::changeState)

    private fun sameKind(initial: Scalar?, value: Scalar) = initial != null && initial::class == value::class

    fun start(document: Document, index: Int = 0, step: Int = 0, state: Map<String, Scalar> = emptyMap(), restoring: Boolean = false, seed: Int = freshSeed()): Session {
        val base = Session(0, 0, document.state, emptyList(), rng = seed)
        val known = state.filter { (name, value) -> sameKind(document.state[name], value) }
        val placed = place(document, base, index, step)
        val session = placed.copy(state = placed.state + known)
        return if (restoring) session else arrive(document, session)
    }

    fun indexOf(document: Document, id: String) = document.pages.indexOfFirst { it.id == id }

    private fun linear(document: Document, from: Int, direction: Int): Int {
        var index = from + direction
        while (index in document.pages.indices) {
            if (!document.pages[index].hidden) return index
            index += direction
        }
        return -1
    }

    fun goTo(document: Document, session: Session, target: String): Session {
        val index = indexOf(document, target)
        if (index !in document.pages.indices) return session
        val entry = HistoryEntry(session.index, session.step, locals(document, session.index, session.state))
        return arrive(document, place(document, session, index, 0)).copy(history = session.history + entry)
    }

    fun back(document: Document, session: Session): Session {
        val entry = session.history.lastOrNull() ?: return advance(document, session, -1)
        return place(document, session, entry.index, entry.step, entry.locals).copy(history = session.history.dropLast(1))
    }

    fun advance(document: Document, session: Session, direction: Int): Session {
        val page = document.pages.getOrNull(session.index) ?: return session
        if (!page.locked) return travel(document, session, direction)
        if (direction == 1 && session.step < page.steps) return session.copy(step = session.step + 1)
        if (direction == -1 && session.step > 0) return session.copy(step = session.step - 1)
        return session
    }

    private fun travel(document: Document, session: Session, direction: Int): Session {
        val page = document.pages.getOrNull(session.index) ?: return session
        if (direction == 1 && session.step < page.steps) return session.copy(step = session.step + 1)
        if (direction == -1 && session.step > 0) return session.copy(step = session.step - 1)
        if (page.hidden && session.history.isNotEmpty()) return back(document, session)
        if (direction == 1 && page.next != null) {
            val index = indexOf(document, page.next)
            if (index >= 0) return arrive(document, place(document, session, index, 0))
        }
        val index = linear(document, session.index, direction)
        if (index < 0) return session
        return arrive(document, place(document, session, index, if (direction == 1) 0 else document.pages[index].steps))
    }

    fun run(document: Document, session: Session, actions: List<Action>): Session = actions.fold(session) { current, action ->
        when (action) {
            is Action.Go -> goTo(document, current, action.target)
            Action.Back -> back(document, current)
            Action.Next -> travel(document, current, 1)
            Action.Prev -> travel(document, current, -1)
            else -> changeState(current, action)
        }
    }

    fun tick(document: Document, session: Session): Session {
        val page = document.pages.getOrNull(session.index) ?: return session
        if (!page.clock) return session
        val ticked = session.copy(elapsed = session.elapsed + 1)
        val timeout = page.timeout ?: return ticked
        val reached = session.elapsed < timeout && ticked.elapsed >= timeout
        return if (reached && page.onTimeout.isNotEmpty()) run(document, ticked, page.onTimeout) else ticked
    }
}
