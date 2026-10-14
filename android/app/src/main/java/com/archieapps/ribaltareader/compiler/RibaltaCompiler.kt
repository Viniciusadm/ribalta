package com.archieapps.ribaltareader.compiler

import com.archieapps.ribaltareader.ir.Compilation
import com.archieapps.ribaltareader.ir.IrParser

object RibaltaCompiler {
    init {
        System.loadLibrary("ribalta_jni")
    }

    @JvmStatic
    private external fun compileBytes(prelude: ByteArray, source: ByteArray, english: Int, options: ByteArray): ByteArray

    const val SLIDE_FORMAT = """{"page":{"width":1920,"height":1080}}"""

    fun compileJson(source: String, english: Boolean = false, prelude: String = "", options: String = SLIDE_FORMAT): String =
        compileBytes(prelude.toByteArray(), source.toByteArray(), if (english) 1 else 0, options.toByteArray()).toString(Charsets.UTF_8)

    fun compile(source: String, english: Boolean = false, prelude: String = ""): Compilation = IrParser.compilation(compileJson(source, english, prelude))
}
