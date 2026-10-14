package com.archieapps.ribalta.lesson

import com.archieapps.ribalta.ir.Block
import com.archieapps.ribalta.ir.BlockKind
import com.archieapps.ribalta.ir.Document
import com.archieapps.ribalta.ir.Inline
import com.archieapps.ribalta.ir.ListItem
import com.archieapps.ribalta.ir.Page

data class TargetOption(val target: LessonTarget, val label: String, val pageTitle: String?)

private const val MAX_LABEL = 60

fun Document.targets(): List<TargetOption> {
    val options = mutableListOf<TargetOption>()
    val seen = mutableSetOf<String>()
    for (page in pages) {
        val title = pageTitle(page)
        options += TargetOption(LessonTarget.Page(page.id), title ?: page.id, title)
        for ((id, label) in anchoredLabels(page)) {
            if (seen.add(id)) options += TargetOption(LessonTarget.Anchor(id), label ?: id, title)
        }
    }
    for ((id, index) in anchors) {
        if (seen.add(id)) options += TargetOption(LessonTarget.Anchor(id), id, pages.getOrNull(index)?.let(::pageTitle))
    }
    return options
}

private fun pageTitle(page: Page): String? =
    firstHeading(page.children)?.let(::truncated)?.takeIf { it.isNotEmpty() }

private fun firstHeading(blocks: List<Block>): String? {
    for (block in blocks) {
        val found = when (val kind = block.kind) {
            is BlockKind.Heading -> plain(kind.content)
            is BlockKind.Layout -> firstHeading(kind.children)
            is BlockKind.Quote -> firstHeading(kind.children)
            else -> null
        }
        if (!found.isNullOrBlank()) return found
    }
    return null
}

private fun anchoredLabels(page: Page): List<Pair<String, String?>> {
    val out = mutableListOf<Pair<String, String?>>()
    collect(listOfNotNull(page.header) + page.children + listOfNotNull(page.footer), out)
    return out
}

private fun collect(blocks: List<Block>, out: MutableList<Pair<String, String?>>) {
    for (block in blocks) {
        block.id?.let { out += it to blockLabel(block) }
        when (val kind = block.kind) {
            is BlockKind.Layout -> collect(kind.children, out)
            is BlockKind.Quote -> collect(kind.children, out)
            is BlockKind.Each -> collect(kind.children, out)
            is BlockKind.BulletList -> kind.items.forEach { collectItem(it, out) }
            else -> Unit
        }
    }
}

private fun collectItem(item: ListItem, out: MutableList<Pair<String, String?>>) {
    item.id?.let { out += it to truncated(plain(item.content)).takeIf { text -> text.isNotEmpty() } }
    collect(item.children, out)
}

private fun blockLabel(block: Block): String? {
    val text = when (val kind = block.kind) {
        is BlockKind.Heading -> plain(kind.content)
        is BlockKind.Paragraph -> plain(kind.content)
        is BlockKind.BulletList -> kind.items.firstNotNullOfOrNull { plain(it.content).takeIf(String::isNotBlank) } ?: ""
        is BlockKind.Quote -> kind.children.firstNotNullOfOrNull(::blockLabel) ?: ""
        is BlockKind.Layout -> kind.children.firstNotNullOfOrNull(::blockLabel) ?: ""
        is BlockKind.Image -> kind.alt
        is BlockKind.Code -> kind.lines.firstOrNull()?.joinToString("") { it.text } ?: ""
        else -> ""
    }
    return truncated(text).takeIf { it.isNotEmpty() }
}

private fun plain(nodes: List<Inline>): String = nodes.joinToString("") { node ->
    when (node) {
        is Inline.Text -> node.value
        is Inline.Code -> node.value
        is Inline.Strong -> plain(node.children)
        is Inline.Emphasis -> plain(node.children)
        is Inline.Strike -> plain(node.children)
        is Inline.Link -> plain(node.children)
        is Inline.Span -> plain(node.children)
        is Inline.Image -> node.alt
        is Inline.Break -> " "
        is Inline.Field, is Inline.Value -> ""
    }
}

private fun truncated(text: String): String {
    val collapsed = text.trim().replace(Regex("\\s+"), " ")
    if (collapsed.length <= MAX_LABEL) return collapsed
    val cut = collapsed.take(MAX_LABEL)
    val atWord = cut.lastIndexOf(' ').takeIf { it > MAX_LABEL / 2 } ?: MAX_LABEL
    return cut.take(atWord).trimEnd() + "…"
}
