package com.archieapps.ribalta.lesson

import com.archieapps.ribalta.compiler.RibaltaCompiler
import com.archieapps.ribalta.ir.Diagnostic
import com.archieapps.ribalta.ir.Document
import com.archieapps.ribalta.ir.IrParser

data class CompileResult(val document: Document, val diagnostics: List<Diagnostic>)

fun compileLesson(source: String, widthPx: Int): CompileResult {
    val options = """{"page":{"width":$widthPx,"height":null},"profile":"lesson"}"""
    val compilation = IrParser.compilation(RibaltaCompiler.compileJson(source, options = options))
    return CompileResult(compilation.document, compilation.diagnostics)
}
