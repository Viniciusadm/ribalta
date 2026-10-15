package com.archieapps.ribalta.ir

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.doubleOrNull
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

object IrParser {
    fun compilation(json: String): Compilation {
        val root = Json.parseToJsonElement(json).jsonObject
        return Compilation(document(root.obj("document")), root.array("diagnostics").map { diagnostic(it.jsonObject) })
    }

    fun document(json: JsonObject) = Document(
        meta = json.obj("meta").let { Meta(it.stringOrNull("title"), it.stringOrNull("mode") == "game", it.boolOrNull("random") == true) },
        theme = theme(json.obj("theme")),
        state = stateMap(json.objOrNull("state")),
        icons = json.objOrNull("icons")?.mapValues { (_, elements) ->
            elements.jsonArray.map { element -> element.jsonObject.let { IconElement(it.string("tag"), it.obj("attrs").mapValues { (_, value) -> value.jsonPrimitive.content }) } }
        } ?: emptyMap(),
        pages = json.array("pages").map { page(it.jsonObject) },
        persist = json.arrayOrNull("persist")?.map { it.jsonPrimitive.content } ?: emptyList(),
        anchors = json.objOrNull("anchors")?.mapValues { (_, page) -> page.jsonPrimitive.doubleOrNull?.toInt() ?: 0 } ?: emptyMap(),
    )

    private fun diagnostic(json: JsonObject) = Diagnostic(json.string("severity"), json.string("code"), json.int("line"), json.string("message"))

    fun color(value: String): Long {
        val digits = value.removePrefix("#")
        val rgb = digits.substring(0, 6).toLong(16)
        val alpha = if (digits.length >= 8) digits.substring(6, 8).toLong(16) else 0xff
        return (alpha shl 24) or rgb
    }

    fun scalar(value: JsonElement): Scalar {
        if (value is JsonArray && value.any { it is JsonObject }) return Scalar.Records(value.map { row -> row.jsonObject.mapValues { (_, field) -> scalar(field) } })
        if (value is JsonArray) return Scalar.List(value.map { it.jsonPrimitive.content })
        val primitive = value.jsonPrimitive
        return when {
            primitive.isString -> Scalar.Text(primitive.content)
            primitive.booleanOrNull != null -> Scalar.Bool(primitive.booleanOrNull!!)
            else -> Scalar.Number(primitive.doubleOrNull ?: 0.0)
        }
    }

    fun stateMap(json: JsonObject?): Map<String, Scalar> = json?.mapValues { (_, value) -> scalar(value) } ?: emptyMap()

    private fun length(json: JsonObject): Length = when (json.string("unit")) {
        "px" -> Length.Px(json.float("value"))
        "pct" -> Length.Pct(json.float("value"))
        else -> Length.Auto
    }

    private fun paint(json: JsonObject): Paint = when (json.string("kind")) {
        "linear" -> Paint.Linear(json.float("angle"), json.array("stops").map { stop -> stop.jsonObject.let { GradientStop(color(it.string("color")), it.float("at")) } })
        else -> Paint.Solid(color(json.string("color")))
    }

    private fun imageSource(json: JsonObject): ImageSource = when (json.string("kind")) {
        "asset" -> ImageSource.Asset(json.string("id"))
        else -> ImageSource.Url(json.string("url"))
    }

    private fun <T : Enum<T>> enumOf(values: Array<T>, name: String?): T? = name?.let { value -> values.firstOrNull { it.name.equals(value, ignoreCase = true) } }

    fun style(json: JsonObject?): Style {
        if (json == null) return Style()
        return Style(
            color = json.stringOrNull("color")?.let(::color),
            bg = json.objOrNull("bg")?.let(::paint),
            bgImage = json.objOrNull("bgImage")?.let(::imageSource),
            font = json.stringOrNull("font"),
            size = json.floatOrNull("size"),
            weight = json.floatOrNull("weight")?.toInt(),
            italic = json.boolOrNull("italic"),
            uppercase = json.boolOrNull("uppercase"),
            underline = json.boolOrNull("underline"),
            align = enumOf(HAlign.entries.toTypedArray(), json.stringOrNull("align")),
            valign = enumOf(VAlign.entries.toTypedArray(), json.stringOrNull("valign")),
            pad = json.arrayOrNull("pad")?.let { it.floatAt(0) to it.floatAt(1) },
            gap = json.floatOrNull("gap"),
            radius = json.floatOrNull("radius"),
            border = json.objOrNull("border")?.let { Border(it.float("width"), color(it.string("color"))) },
            shadow = enumOf(ShadowSize.entries.toTypedArray(), json.stringOrNull("shadow")),
            w = json.objOrNull("w")?.let(::length),
            h = json.objOrNull("h")?.let(::length),
            x = json.objOrNull("x")?.let(::length),
            y = json.objOrNull("y")?.let(::length),
            opacity = json.floatOrNull("opacity"),
            leading = json.floatOrNull("leading"),
            tracking = json.floatOrNull("tracking"),
            shrink = json.floatOrNull("shrink"),
            enter = enumOf(EnterKind.entries.toTypedArray(), json.stringOrNull("enter")),
        )
    }

    fun expr(json: JsonObject): Expr = when (json.string("op")) {
        "lit" -> Expr.Lit(scalar(json.getValue("value")))
        "state" -> Expr.State(json.string("name"))
        "item" -> Expr.Item(json.string("name"), json.stringOrNull("field"))
        "step" -> Expr.Step
        "elapsed" -> Expr.Elapsed
        "hover" -> Expr.Hover
        "not" -> Expr.Not(expr(json.obj("arg")))
        "neg" -> Expr.Neg(expr(json.obj("arg")))
        "binary" -> Expr.Binary(json.string("operator"), expr(json.obj("left")), expr(json.obj("right")))
        "cond" -> Expr.Cond(expr(json.obj("test")), expr(json.obj("then")), expr(json.obj("otherwise")))
        "call" -> Expr.Call(enumOf(Builtin.entries.toTypedArray(), json.string("name")) ?: Builtin.Round, json.array("args").map { expr(it.jsonObject) })
        else -> Expr.Lit(Scalar.Bool(false))
    }

    fun action(json: JsonObject): Action? = when (json.string("type")) {
        "set" -> Action.Set(json.string("name"), expr(json.obj("value")))
        "toggle" -> Action.Toggle(json.string("name"))
        "add" -> Action.Add(json.string("name"), expr(json.obj("value")))
        "remove" -> Action.Remove(json.string("name"), expr(json.obj("value")))
        "clear" -> Action.Clear(json.string("name"))
        "play" -> Action.Play(json.string("name"))
        "stop" -> Action.Stop(json.string("name"))
        "go" -> Action.Go(json.string("target"))
        "back" -> Action.Back
        "next" -> Action.Next
        "prev" -> Action.Prev
        else -> null
    }

    fun actions(json: JsonArray?): List<Action> = json?.mapNotNull { action(it.jsonObject) } ?: emptyList()

    private fun behavior(json: JsonObject) = Behavior(
        whenExpr = json.objOrNull("when")?.let(::expr),
        variants = json.arrayOrNull("variants")?.map { variant -> variant.jsonObject.let { Variant(expr(it.obj("when")), style(it.objOrNull("style"))) } } ?: emptyList(),
        click = actions(json.arrayOrNull("click")),
        key = json.stringOrNull("key"),
    )

    private fun inlines(json: JsonArray?): List<Inline> = json?.map { inline(it.jsonObject) } ?: emptyList()

    private fun inline(json: JsonObject): Inline = when (json.string("type")) {
        "text" -> Inline.Text(json.string("value"))
        "strong" -> Inline.Strong(inlines(json.arrayOrNull("children")))
        "emphasis" -> Inline.Emphasis(inlines(json.arrayOrNull("children")))
        "strike" -> Inline.Strike(inlines(json.arrayOrNull("children")))
        "code" -> Inline.Code(json.string("value"))
        "link" -> Inline.Link(json.string("href"), inlines(json.arrayOrNull("children")))
        "span" -> Inline.Span(style(json.objOrNull("style")), json.intOrNull("step"), behavior(json), inlines(json.arrayOrNull("children")))
        "image" -> Inline.Image(imageSource(json.obj("src")), json.stringOrNull("alt") ?: "", style(json.objOrNull("style")))
        "field" -> Inline.Field(json.string("name"))
        "expr" -> Inline.Value(expr(json.obj("expr")))
        else -> Inline.Break
    }

    private fun points(json: JsonArray) = json.map { point -> point.jsonArray.let { it.floatAt(0) to it.floatAt(1) } }

    private fun ink(json: JsonObject): Ink = when (json.string("kind")) {
        "text" -> Ink.Text
        "background" -> Ink.Background
        else -> Ink.Color(color(json.string("color")))
    }

    private fun shape(json: JsonObject): Shape? = when (json.string("shape")) {
        "rect" -> Shape.Rect(json.float("x"), json.float("y"), json.float("w"), json.float("h"), json.float("radius"), ink(json.obj("ink")), json.float("opacity"))
        "line" -> Shape.Line(json.float("x1"), json.float("y1"), json.float("x2"), json.float("y2"), json.float("width"), ink(json.obj("ink")), json.float("opacity"))
        "polyline" -> Shape.Polyline(points(json.array("points")), json.float("width"), ink(json.obj("ink")), json.float("opacity"))
        "polygon" -> Shape.Polygon(points(json.array("points")), ink(json.obj("ink")), json.float("opacity"))
        "circle" -> Shape.Circle(json.float("cx"), json.float("cy"), json.float("r"), ink(json.obj("ink")), json.float("opacity"))
        "sector" -> Shape.Sector(json.float("cx"), json.float("cy"), json.float("outer"), json.float("inner"), json.float("start"), json.float("end"), ink(json.obj("ink")), ink(json.obj("stroke")), json.float("strokeWidth"))
        "text" -> Shape.Text(
            json.float("x"), json.float("y"), json.string("text"),
            enumOf(TextAnchor.entries.toTypedArray(), json.string("anchor")) ?: TextAnchor.Start,
            json.string("baseline") == "middle", json.float("size"), json.int("weight"), ink(json.obj("ink")), json.float("opacity"),
        )
        else -> null
    }

    private fun listItem(json: JsonObject) = ListItem(
        content = inlines(json.arrayOrNull("content")),
        children = json.arrayOrNull("children")?.map { block(it.jsonObject) } ?: emptyList(),
        style = style(json.objOrNull("style")),
        step = json.intOrNull("step"),
        behavior = behavior(json),
        id = json.stringOrNull("id"),
    )

    fun block(json: JsonObject): Block {
        val children = { json.arrayOrNull("children")?.map { block(it.jsonObject) } ?: emptyList() }
        val kind: BlockKind = when (json.string("type")) {
            "heading" -> BlockKind.Heading(json.int("level"), inlines(json.arrayOrNull("content")))
            "paragraph" -> BlockKind.Paragraph(inlines(json.arrayOrNull("content")))
            "list" -> BlockKind.BulletList(json.boolOrNull("ordered") == true, json.intOrNull("start") ?: 1, json.array("items").map { listItem(it.jsonObject) })
            "quote" -> BlockKind.Quote(children())
            "each" -> BlockKind.Each(expr(json.obj("list")), json.string("item"), json.stringOrNull("index"), children())
            "code" -> BlockKind.Code(json.array("lines").map { line -> line.jsonArray.map { token -> token.jsonObject.let { Token(it.string("kind"), it.string("text")) } } })
            "table" -> BlockKind.Table(
                json.array("header").map { inlines(it.jsonArray) },
                json.array("aligns").map { enumOf(ColumnAlign.entries.toTypedArray(), it.jsonPrimitive.content) ?: ColumnAlign.Left },
                json.array("rows").map { row -> row.jsonArray.map { inlines(it.jsonArray) } },
            )
            "image" -> BlockKind.Image(imageSource(json.obj("src")), json.stringOrNull("alt") ?: "", if (json.stringOrNull("fit") == "cover") ImageFit.Cover else ImageFit.Contain)
            "layout" -> BlockKind.Layout(
                enumOf(LayoutKind.entries.toTypedArray(), json.string("layout")) ?: LayoutKind.Stack,
                json.intOrNull("cols"),
                json.arrayOrNull("widths")?.map { it.jsonPrimitive.doubleOrNull?.toFloat() ?: 1f },
                json.boolOrNull("wrap") == true,
                children(),
            )
            "chart" -> json.obj("scene").let { BlockKind.Chart(ChartScene(it.float("width"), it.float("height"), it.array("shapes").mapNotNull { shape -> shape(shape.jsonObject) })) }
            "icon" -> BlockKind.Icon(json.string("name"))
            "spacer" -> BlockKind.Spacer(json.float("size"))
            "divider" -> BlockKind.Divider
            "input" -> BlockKind.Input(json.string("name"), json.boolOrNull("numeric") == true, json.stringOrNull("placeholder"), actions(json.arrayOrNull("onSubmit")))
            "sound" -> BlockKind.Sound(json.stringOrNull("name"), imageSource(json.obj("src")), json.boolOrNull("autoplay") == true, json.boolOrNull("loop") == true, json.floatOrNull("volume"))
            else -> BlockKind.Spacer(0f)
        }
        return Block(kind, style(json.objOrNull("style")), json.intOrNull("step"), behavior(json), json.stringOrNull("id"))
    }

    fun page(json: JsonObject) = Page(
        id = json.string("id"),
        width = (json["width"] as? JsonPrimitive)?.doubleOrNull?.toFloat() ?: 1920f,
        height = (json["height"] as? JsonPrimitive)?.doubleOrNull?.toFloat(),
        steps = json.intOrNull("steps") ?: 0,
        style = style(json.objOrNull("style")),
        children = json.arrayOrNull("children")?.map { block(it.jsonObject) } ?: emptyList(),
        header = json.objOrNull("header")?.let(::block),
        footer = json.objOrNull("footer")?.let(::block),
        overlay = json.objOrNull("overlay")?.let(::block),
        hidden = json.boolOrNull("hidden") == true,
        next = json.stringOrNull("next"),
        state = stateMap(json.objOrNull("state")),
        transition = enumOf(TransitionKind.entries.toTypedArray(), json.stringOrNull("transition")),
        onEnter = actions(json.arrayOrNull("onEnter")),
        locked = json.boolOrNull("locked") == true,
        timeout = (json["timeout"] as? JsonPrimitive)?.doubleOrNull,
        onTimeout = actions(json.arrayOrNull("onTimeout")),
        clock = json.boolOrNull("clock") == true,
        shrinks = json.toString().contains("\"shrink\":"),
    )

    private fun timing(json: JsonObject) = Timing(json.int("duration"), json.array("easing").map { it.jsonPrimitive.doubleOrNull?.toFloat() ?: 0f })

    private fun animation(json: JsonObject) = Animation(
        timing(json),
        json.obj("from").let { Pose(it.float("opacity"), it.objOrNull("dx")?.let(::length), it.objOrNull("dy")?.let(::length), it.floatOrNull("scale")) },
    )

    private fun motion(json: JsonObject) = Motion(
        enter = json.obj("enter").entries.mapNotNull { (name, value) -> enumOf(EnterKind.entries.toTypedArray(), name)?.let { it to animation(value.jsonObject) } }.toMap(),
        transitions = json.obj("transitions").entries.mapNotNull { (name, value) -> enumOf(TransitionKind.entries.toTypedArray(), name)?.let { it to animation(value.jsonObject) } }.toMap(),
        reveal = timing(json.obj("reveal")),
        variant = timing(json.obj("variant")),
    )

    private fun pair(json: JsonObject, key: String) = json.array(key).let { it.floatAt(0) to it.floatAt(1) }

    private fun elements(json: JsonObject): Elements {
        val shadows = json.obj("shadows")
        val list = json.obj("list")
        val quote = json.obj("quote")
        val code = json.obj("code")
        val inlineCode = json.obj("inlineCode")
        val link = json.obj("link")
        val table = json.obj("table")
        val divider = json.obj("divider")
        val icon = json.obj("icon")
        return Elements(
            shadows = shadows.entries.mapNotNull { (name, value) ->
                enumOf(ShadowSize.entries.toTypedArray(), name)?.let { size -> value.jsonObject.let { size to ShadowSpec(it.float("x"), it.float("y"), it.float("blur"), color(it.string("color"))) } }
            }.toMap(),
            list = ListSpec(list.float("gapEm"), list.float("indentEm"), color(list.string("markerColor")), list.int("markerWeight"), list.float("nestedGapEm"), list.float("nestedScale")),
            quote = QuoteSpec(quote.float("gapEm"), quote.float("indent"), quote.float("barWidth"), color(quote.string("barColor")), quote.boolOrNull("italic") == true),
            code = CodeSpec(
                code.string("font"), code.float("size"), code.float("leading"), pair(code, "pad"), code.float("radius"), color(code.string("bg")), color(code.string("color")),
                code.obj("syntax").mapValues { (_, value) -> value.jsonObject.let { TokenStyle(color(it.string("color")), it.boolOrNull("italic") == true) } },
            ),
            inlineCode = InlineCodeSpec(inlineCode.string("font"), inlineCode.float("scale"), pair(inlineCode, "padEm"), inlineCode.float("radiusEm"), inlineCode.float("tint")),
            link = LinkSpec(color(link.string("color")), link.float("underlineEm"), link.float("underlineOffsetEm")),
            table = TableSpec(table.float("size"), pair(table, "cellPad"), table.float("ruleWidth"), table.float("headerRuleWidth"), color(table.string("ruleColor")), table.int("headerWeight")),
            divider = DividerSpec(divider.float("thickness"), divider.float("radius"), color(divider.string("color"))),
            icon = IconSpec(icon.float("size"), icon.float("stroke")),
        )
    }

    private fun layoutDefaults(json: JsonObject): LayoutDefaults {
        val gaps = json.obj("gaps")
        val band = json.obj("band")
        return LayoutDefaults(
            gaps = gaps.entries.mapNotNull { (name, value) -> enumOf(LayoutKind.entries.toTypedArray(), name)?.let { it to (value.jsonPrimitive.doubleOrNull?.toFloat() ?: 0f) } }.toMap(),
            gridColumns = json.int("gridColumns"),
            band = Band(band.float("gap"), band.float("edgeScale"), band.float("heightScale")),
            chartMaxHeight = json.float("chartMaxHeight"),
        )
    }

    private fun theme(json: JsonObject) = Theme(
        colors = json.obj("colors").mapValues { (_, value) -> color(value.jsonPrimitive.content) },
        fontFamilies = json.array("fontFamilies").map { family -> family.jsonObject.let { FontFamilySpec(it.string("family"), it.array("weights").map { weight -> weight.jsonPrimitive.content.toDouble().toInt() }) } },
        text = style(json.objOrNull("text")),
        headings = json.array("headings").map { style(it.jsonObject) },
        page = style(json.objOrNull("page")),
        motion = motion(json.obj("motion")),
        elements = elements(json.obj("elements")),
        layout = layoutDefaults(json.obj("layout")),
    )
}

private fun JsonObject.obj(key: String) = getValue(key).jsonObject
private fun JsonObject.objOrNull(key: String) = (get(key) as? JsonObject)
private fun JsonObject.array(key: String) = getValue(key).jsonArray
private fun JsonObject.arrayOrNull(key: String) = (get(key) as? JsonArray)
private fun JsonObject.string(key: String) = getValue(key).jsonPrimitive.content
private fun JsonObject.stringOrNull(key: String) = get(key)?.takeUnless { it is JsonNull }?.jsonPrimitive?.content
private fun JsonObject.floatOrNull(key: String) = (get(key) as? JsonPrimitive)?.doubleOrNull?.toFloat()
private fun JsonObject.float(key: String) = floatOrNull(key) ?: 0f
private fun JsonObject.intOrNull(key: String) = (get(key) as? JsonPrimitive)?.doubleOrNull?.toInt()
private fun JsonObject.int(key: String) = intOrNull(key) ?: 0
private fun JsonObject.boolOrNull(key: String) = (get(key) as? JsonPrimitive)?.booleanOrNull
private fun JsonArray.floatAt(index: Int) = (getOrNull(index) as? JsonPrimitive)?.doubleOrNull?.toFloat() ?: 0f
