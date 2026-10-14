package com.archieapps.ribaltareader

import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.BackHandler
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.focusable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.key
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isAltPressed
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.isMetaPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.input.key.utf16CodePoint
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import com.archieapps.ribalta.compiler.RibaltaCompiler
import com.archieapps.ribalta.ir.Compilation
import com.archieapps.ribalta.render.Fonts
import com.archieapps.ribalta.render.Images
import com.archieapps.ribalta.render.Interaction
import com.archieapps.ribalta.render.LocalInteraction
import com.archieapps.ribalta.render.RenderContext
import com.archieapps.ribalta.render.ScaledPage
import com.archieapps.ribalta.render.PageSounds
import com.archieapps.ribalta.render.PageView
import com.archieapps.ribalta.render.entrance
import com.archieapps.ribalta.render.pageBackground
import com.archieapps.ribalta.runtime.Navigation
import com.archieapps.ribalta.runtime.Persist
import com.archieapps.ribalta.runtime.Shortcuts
import com.archieapps.ribalta.runtime.Scope
import com.archieapps.ribalta.runtime.Session
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.util.Locale

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            MaterialTheme(colorScheme = darkColorScheme()) { Reader(::immersive) }
        }
    }

    private fun immersive(enabled: Boolean) {
        val controller = WindowCompat.getInsetsController(window, window.decorView)
        if (enabled) {
            controller.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
            controller.hide(WindowInsetsCompat.Type.systemBars())
        } else {
            controller.show(WindowInsetsCompat.Type.systemBars())
        }
    }
}

private class Opened(val compilation: Compilation, val source: String)

@Composable
private fun Reader(immersive: (Boolean) -> Unit) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val english = remember { Locale.getDefault().language != "pt" }
    val fonts = remember { Fonts(context.applicationContext) }
    val images = remember { Images(null) }
    var opened by remember { mutableStateOf<Opened?>(null) }
    var loading by remember { mutableStateOf(false) }
    var failure by remember { mutableStateOf<String?>(null) }

    fun open(source: String, read: () -> String) {
        loading = true
        failure = null
        scope.launch {
            runCatching {
                val compilation = withContext(Dispatchers.Default) { RibaltaCompiler.compile(read(), english) }
                fonts.load(compilation.document.theme.fontFamilies)
                compilation
            }.onSuccess {
                opened = Opened(it, source)
                immersive(true)
            }.onFailure { failure = it.message ?: it.toString() }
            loading = false
        }
    }

    val picker = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri: Uri? ->
        if (uri != null) open(uri.toString()) { context.contentResolver.openInputStream(uri)!!.use { it.readBytes().toString(Charsets.UTF_8) } }
    }

    val current = opened
    if (current != null) {
        BackHandler {
            opened = null
            immersive(false)
        }
        DocumentViewer(current.compilation, Persist(context.applicationContext, current.source), fonts, images, english)
        return
    }

    Box(Modifier.fillMaxSize().background(Color(0xff0f141b)).padding(32.dp), contentAlignment = Alignment.Center) {
        Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(16.dp)) {
            Text("Ribalta Reader", color = Color.White, fontSize = 32.sp)
            Text(if (english) "Open a .ribalta file to view it." else "Abra um arquivo .ribalta para visualizar.", color = Color(0xff8a97a8))
            if (loading) {
                CircularProgressIndicator()
            } else {
                Button(onClick = { picker.launch(arrayOf("*/*")) }) { Text(if (english) "Open .ribalta file" else "Abrir arquivo .ribalta") }
                OutlinedButton(onClick = { open("sample") { context.assets.open("samples/showcase.ribalta").use { it.readBytes().toString(Charsets.UTF_8) } } }) {
                    Text(if (english) "Open sample" else "Abrir exemplo")
                }
            }
            failure?.let { Text(it, color = Color(0xffff8a80)) }
        }
    }
}

@Composable
private fun DocumentViewer(compilation: Compilation, persist: Persist, fonts: Fonts, images: Images, english: Boolean) {
    val document = compilation.document
    var session by remember(document) { mutableStateOf<Session>(Navigation.start(document, state = persist.load(document))) }
    val focus = remember { FocusRequester() }
    var showErrors by remember(document) { mutableStateOf(compilation.diagnostics.any { it.severity == "error" }) }
    if (document.pages.isEmpty()) {
        Box(Modifier.fillMaxSize().background(Color.Black), contentAlignment = Alignment.Center) {
            Text(if (english) "This file has no pages." else "Este arquivo não tem páginas.", color = Color.White)
        }
        return
    }
    val page = document.pages[session.index]
    fun shortcut(key: String): Boolean {
        val actions = Shortcuts.forPage(document, session)[key] ?: return false
        session = Navigation.run(document, session, actions)
        return true
    }
    val interaction = Interaction(Scope(session.state, session.step, session.elapsed), shortcut = ::shortcut) { actions -> session = Navigation.run(document, session, actions) }
    LaunchedEffect(document, session.index, page.clock) {
        if (!page.clock) return@LaunchedEffect
        while (true) {
            delay(1000)
            session = Navigation.tick(document, session)
        }
    }
    LaunchedEffect(document, session.state) { persist.save(document, session.state) }
    LaunchedEffect(Unit) { runCatching { focus.requestFocus() } }
    PageSounds(page, session.cues, images)
    val render = RenderContext(document.theme, fonts, document.icons, images, pageBackground(document, page), english)
    Box(
        Modifier.fillMaxSize().background(Color.Black).pointerInput(document) {
            detectTapGestures { offset ->
                session = Navigation.advance(document, session, if (offset.x < size.width / 3f) -1 else 1)
            }
        }.focusRequester(focus).focusable().onKeyEvent { event ->
            if (event.type != KeyEventType.KeyDown || event.isCtrlPressed || event.isAltPressed || event.isMetaPressed) return@onKeyEvent false
            val name = keyName(event)
            if (name != null && shortcut(name)) return@onKeyEvent true
            val direction = when (event.key) {
                Key.DirectionRight, Key.DirectionDown, Key.Spacebar, Key.PageDown -> 1
                Key.DirectionLeft, Key.DirectionUp, Key.PageUp -> -1
                else -> return@onKeyEvent false
            }
            session = Navigation.advance(document, session, direction)
            true
        },
    ) {
        ScaledPage(page.width, page.height, Modifier.fillMaxSize()) {
            key(session.index) {
                Box(Modifier.entrance(page.transition?.let { document.theme.motion.transitions[it] })) {
                    CompositionLocalProvider(LocalInteraction provides interaction) { PageView(document, page, render) }
                }
            }
        }
        if (showErrors) {
            val errors = compilation.diagnostics.filter { it.severity == "error" }.take(3)
            Column(
                Modifier.align(Alignment.BottomCenter).fillMaxWidth().background(Color(0xdd3b0d0d)).padding(12.dp).pointerInput(Unit) { detectTapGestures { showErrors = false } },
            ) {
                for (error in errors) Text("${error.line}: ${error.message}", color = Color.White, fontSize = 13.sp)
            }
        }
    }
}

private fun keyName(event: KeyEvent): String? = when (event.key) {
    Key.Spacebar -> "Space"
    Key.Enter, Key.NumPadEnter -> "Enter"
    Key.DirectionUp -> "ArrowUp"
    Key.DirectionDown -> "ArrowDown"
    Key.DirectionLeft -> "ArrowLeft"
    Key.DirectionRight -> "ArrowRight"
    else -> event.utf16CodePoint.toChar().lowercaseChar().takeIf { it in 'a'..'z' || it in '0'..'9' }?.toString()
}
