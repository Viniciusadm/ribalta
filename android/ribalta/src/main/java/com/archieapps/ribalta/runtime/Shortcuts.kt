package com.archieapps.ribalta.runtime

import com.archieapps.ribalta.ir.Action
import com.archieapps.ribalta.ir.Behavior
import com.archieapps.ribalta.ir.Block
import com.archieapps.ribalta.ir.BlockKind
import com.archieapps.ribalta.ir.Document
import com.archieapps.ribalta.ir.Inline

object Shortcuts {
    fun forPage(document: Document, session: Session): Map<String, List<Action>> {
        val page = document.pages.getOrNull(session.index) ?: return emptyMap()
        val scope = Scope(document.state + page.state + session.state, session.step, session.elapsed)
        val found = linkedMapOf<String, List<Action>>()
        fun visible(behavior: Behavior, step: Int?) = Evaluator.holds(behavior.whenExpr, scope) && (step == null || step <= session.step)
        fun claim(behavior: Behavior) {
            val key = behavior.key ?: return
            if (behavior.click.isNotEmpty() && key !in found) found[key] = behavior.click
        }
        fun inlines(nodes: List<Inline>) {
            for (node in nodes) {
                when (node) {
                    is Inline.Span -> if (visible(node.behavior, node.step)) {
                        claim(node.behavior)
                        inlines(node.children)
                    }
                    is Inline.Strong -> inlines(node.children)
                    is Inline.Emphasis -> inlines(node.children)
                    is Inline.Strike -> inlines(node.children)
                    is Inline.Link -> inlines(node.children)
                    else -> Unit
                }
            }
        }
        fun blocks(list: List<Block>) {
            for (block in list) {
                if (!visible(block.behavior, block.step)) continue
                claim(block.behavior)
                when (val kind = block.kind) {
                    is BlockKind.Heading -> inlines(kind.content)
                    is BlockKind.Paragraph -> inlines(kind.content)
                    is BlockKind.Quote -> blocks(kind.children)
                    is BlockKind.Layout -> blocks(kind.children)
                    is BlockKind.BulletList -> for (item in kind.items) {
                        if (!visible(item.behavior, item.step)) continue
                        claim(item.behavior)
                        inlines(item.content)
                        blocks(item.children)
                    }
                    is BlockKind.Table -> (kind.header + kind.rows.flatten()).forEach(::inlines)
                    else -> Unit
                }
            }
        }
        blocks(listOfNotNull(page.header) + page.children + listOfNotNull(page.footer, page.overlay))
        return found
    }
}
