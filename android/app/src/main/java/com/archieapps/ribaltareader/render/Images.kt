package com.archieapps.ribaltareader.render

import android.graphics.BitmapFactory
import android.util.LruCache
import androidx.compose.runtime.Composable
import androidx.compose.runtime.State
import androidx.compose.runtime.produceState
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import com.archieapps.ribaltareader.ir.ImageSource
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.net.HttpURLConnection
import java.net.URL
import android.util.Base64

class Images(private val assetBase: String?) {
    private val cache = LruCache<String, ImageBitmap>(32)

    fun address(source: ImageSource): String? = when (source) {
        is ImageSource.Url -> source.url
        is ImageSource.Asset -> assetBase?.let { "${it.trimEnd('/')}/api/v1/assets/${source.id}" }
    }

    suspend fun load(source: ImageSource): ImageBitmap? {
        val address = address(source) ?: return null
        cache.get(address)?.let { return it }
        val bitmap = withContext(Dispatchers.IO) {
            runCatching {
                val bytes = if (address.startsWith("data:")) {
                    Base64.decode(address.substringAfter("base64,"), Base64.DEFAULT)
                } else {
                    val connection = URL(address).openConnection() as HttpURLConnection
                    connection.connectTimeout = 10_000
                    connection.readTimeout = 15_000
                    connection.inputStream.use { it.readBytes() }
                }
                BitmapFactory.decodeByteArray(bytes, 0, bytes.size)?.asImageBitmap()
            }.getOrNull()
        } ?: return null
        cache.put(address, bitmap)
        return bitmap
    }
}

@Composable
fun rememberImage(images: Images, source: ImageSource?): State<ImageBitmap?> = produceState<ImageBitmap?>(null, source) {
    value = source?.let { images.load(it) }
}
