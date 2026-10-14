package com.archieapps.ribalta.lesson

sealed interface LessonTarget {
    data class Page(val id: String) : LessonTarget
    data class Anchor(val id: String) : LessonTarget

    companion object
}

private const val PAGE_PREFIX = "page:"
private const val ANCHOR_PREFIX = "anchor:"
private val identifier = Regex("[A-Za-z0-9_-]+")

fun LessonTarget.Companion.parse(raw: String): LessonTarget? = when {
    raw.startsWith(PAGE_PREFIX) -> raw.removePrefix(PAGE_PREFIX).takeIf(identifier::matches)?.let { LessonTarget.Page(it) }
    raw.startsWith(ANCHOR_PREFIX) -> raw.removePrefix(ANCHOR_PREFIX).takeIf(identifier::matches)?.let { LessonTarget.Anchor(it) }
    else -> null
}

fun LessonTarget.serialize(): String = when (this) {
    is LessonTarget.Page -> PAGE_PREFIX + id
    is LessonTarget.Anchor -> ANCHOR_PREFIX + id
}
