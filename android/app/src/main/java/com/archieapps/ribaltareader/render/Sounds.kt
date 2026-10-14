package com.archieapps.ribaltareader.render

import android.content.Context
import android.media.AudioAttributes
import android.media.MediaPlayer
import android.util.Base64
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import com.archieapps.ribaltareader.ir.Block
import com.archieapps.ribaltareader.ir.BlockKind
import com.archieapps.ribaltareader.ir.Page
import com.archieapps.ribaltareader.runtime.Cue
import com.archieapps.ribaltareader.runtime.CueType
import java.io.File

private class Track(context: Context, address: String, private val sound: BlockKind.Sound) {
    private val player = MediaPlayer()
    private var prepared = false
    private var wanted = sound.autoplay
    private var temporary: File? = null

    init {
        runCatching {
            player.setAudioAttributes(AudioAttributes.Builder().setUsage(AudioAttributes.USAGE_MEDIA).setContentType(AudioAttributes.CONTENT_TYPE_SONIFICATION).build())
            if (address.startsWith("data:")) {
                val file = File.createTempFile("ribalta-sound", null, context.cacheDir)
                file.writeBytes(Base64.decode(address.substringAfter("base64,"), Base64.DEFAULT))
                temporary = file
                player.setDataSource(file.absolutePath)
            } else {
                player.setDataSource(address)
            }
            player.isLooping = sound.loop
            (sound.volume ?: 1f).coerceIn(0f, 1f).let { player.setVolume(it, it) }
            player.setOnPreparedListener {
                prepared = true
                if (wanted) start()
            }
            player.prepareAsync()
        }
    }

    fun start() {
        wanted = true
        if (!prepared) return
        runCatching {
            player.seekTo(0)
            player.start()
        }
    }

    fun halt() {
        wanted = false
        if (!prepared) return
        runCatching {
            player.pause()
            player.seekTo(0)
        }
    }

    fun release() {
        runCatching { player.release() }
        temporary?.delete()
    }
}

private fun soundsOf(page: Page): List<BlockKind.Sound> {
    val found = mutableListOf<BlockKind.Sound>()
    fun visit(blocks: List<Block>) {
        for (block in blocks) {
            when (val kind = block.kind) {
                is BlockKind.Sound -> found += kind
                is BlockKind.Layout -> visit(kind.children)
                is BlockKind.Quote -> visit(kind.children)
                is BlockKind.BulletList -> kind.items.forEach { visit(it.children) }
                else -> Unit
            }
        }
    }
    visit(listOfNotNull(page.header) + page.children + listOfNotNull(page.footer, page.overlay))
    return found
}

private class Seen(var id: Int)

@Composable
fun PageSounds(page: Page, cues: List<Cue>, images: Images) {
    val context = LocalContext.current
    val tracks = remember(page) {
        soundsOf(page).mapNotNull { sound -> images.address(sound.src)?.let { sound.name to Track(context, it, sound) } }
    }
    DisposableEffect(tracks) {
        onDispose { tracks.forEach { (_, track) -> track.release() } }
    }
    val seen = remember { Seen(0) }
    LaunchedEffect(cues) {
        for (cue in cues) {
            if (cue.id <= seen.id) continue
            seen.id = cue.id
            val track = tracks.firstOrNull { it.first == cue.name }?.second ?: continue
            if (cue.type == CueType.Play) track.start() else track.halt()
        }
    }
}
