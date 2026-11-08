package com.hyprtv

import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.drawable.Drawable
import android.provider.Settings
import android.util.LruCache
import java.util.concurrent.Executors

/** Una app (o un acceso especial) que aparece como mosaico en el launcher. */
class AppEntry(val pkg: String, val label: String, val intent: Intent, val defaultWorkspace: Int, val special: Boolean = false) {
    val workspace: Int get() = Cfg.workspaceOf(pkg) ?: defaultWorkspace
}

object Apps {
    // Paquetes que, salvo que el usuario diga otra cosa, van al espacio "media" (2) o "sistema" (3).
    private val MEDIA = listOf(
        "youtube", "netflix", "primevideo", "amazon.avod", "disney", "hbo", "spotify", "kodi", "vlc", "plex", "twitch",
        "mgstv", "cast.receiver", "crunchyroll", "paramount", "hulu", "music", "video", "iptv", "player",
    )
    private val SYSTEM = listOf("settings", "vending", "packageinstaller", "filemanager", "files", "browser", "chrome", "miracast", "hyprtv", "launcherx")

    private fun defaultWorkspace(pkg: String): Int = when {
        SYSTEM.any { pkg.contains(it) } -> 3
        MEDIA.any { pkg.contains(it) } -> 2
        else -> 1
    }

    fun load(ctx: Context): List<AppEntry> {
        val pm = ctx.packageManager
        val seen = HashSet<String>()
        val out = ArrayList<AppEntry>()
        for (category in listOf(Intent.CATEGORY_LEANBACK_LAUNCHER, Intent.CATEGORY_LAUNCHER)) {
            val q = Intent(Intent.ACTION_MAIN).addCategory(category)
            for (ri in pm.queryIntentActivities(q, 0)) {
                val pkg = ri.activityInfo.packageName
                if (pkg == ctx.packageName || !seen.add(pkg)) continue
                val intent = Intent(Intent.ACTION_MAIN).addCategory(category).setClassName(pkg, ri.activityInfo.name).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                out.add(AppEntry(pkg, ri.loadLabel(pm).toString(), intent, defaultWorkspace(pkg)))
            }
        }
        out.sortBy { it.label.lowercase() }
        // Accesos que no son apps del launcher: los ajustes de la TV y los de HyprTV (así siempre se llega a ellos).
        out.add(AppEntry("special.settings", "Ajustes", Intent(Settings.ACTION_SETTINGS).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK), 3, special = true))
        out.add(
            AppEntry(
                "special.hyprtv", "HyprTV",
                Intent(ctx, SettingsActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK), 3, special = true,
            ),
        )
        return out
    }
}

/** Imagen de un mosaico: el banner de la app (16:9) o, si no tiene, su icono. */
class TileImage(val bitmap: Bitmap, val isBanner: Boolean)

/**
 * Iconos y banners bajo demanda, en un solo hilo y con un caché pequeño: la TV tiene poca RAM y el launcher no necesita
 * tener cargadas más de unas decenas de imágenes.
 */
class IconLoader(private val ctx: Context) {
    private val cache = LruCache<String, TileImage>(40)
    private val pool = Executors.newSingleThreadExecutor()
    private val ui = android.os.Handler(android.os.Looper.getMainLooper())

    fun clear() = cache.evictAll()

    fun load(entry: AppEntry, w: Int, h: Int, done: (TileImage) -> Unit) {
        cache.get(entry.pkg)?.let { done(it); return }
        pool.execute {
            val img = runCatching { render(entry, w, h) }.getOrNull() ?: return@execute
            cache.put(entry.pkg, img)
            ui.post { done(img) }
        }
    }

    private fun render(entry: AppEntry, w: Int, h: Int): TileImage {
        val pm = ctx.packageManager
        val comp = entry.intent.component
        var drawable: Drawable? = null
        var banner = false
        if (!entry.special) {
            drawable = runCatching { comp?.let { pm.getActivityBanner(it) } }.getOrNull()
            if (drawable != null) banner = true
            if (drawable == null) drawable = runCatching { pm.getApplicationBanner(entry.pkg) }.getOrNull()?.also { banner = true }
        }
        if (drawable == null) {
            drawable = runCatching { if (entry.special && entry.pkg == "special.settings") pm.getActivityIcon(entry.intent) else comp?.let { pm.getActivityIcon(it) } }.getOrNull()
        }
        if (drawable == null) drawable = pm.defaultActivityIcon
        val bmp = Bitmap.createBitmap(w, h, Bitmap.Config.ARGB_8888)
        if (banner) {
            drawable.setBounds(0, 0, w, h)
        } else {
            // Un icono cuadrado: centrado y a la mitad del alto del mosaico.
            val s = (h * 0.55f).toInt()
            drawable.setBounds((w - s) / 2, (h - s) / 2 - h / 12, (w + s) / 2, (h + s) / 2 - h / 12)
        }
        drawable.draw(Canvas(bmp))
        return TileImage(bmp, banner)
    }
}
