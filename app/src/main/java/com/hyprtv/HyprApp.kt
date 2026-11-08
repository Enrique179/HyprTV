package com.hyprtv

import android.app.Application
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.os.SystemClock
import java.io.File

class HyprApp : Application() {
    override fun onCreate() {
        super.onCreate()
        Cfg.init(this)
        Crashes.guard(this)
        val previous = Thread.getDefaultUncaughtExceptionHandler()
        Thread.setDefaultUncaughtExceptionHandler { t, e ->
            Crashes.record(this)
            previous?.uncaughtException(t, e)
        }
    }
}

/**
 * Protección contra un bucle de cierres: si el proceso se cae dos veces en menos de un minuto se apaga el tema solo, y el
 * inicio vuelve a ser el de Google TV hasta que se vuelva a activar a mano.
 */
object Crashes {
    private const val WINDOW_MS = 60_000L
    private const val FILE = "crashes.txt"

    // Hora de pared (no uptime): hay que compararla entre procesos distintos.
    private fun now() = System.currentTimeMillis()

    fun record(ctx: Context) {
        runCatching { File(ctx.filesDir, FILE).appendText("${now()}\n") }
    }

    fun guard(ctx: Context) {
        val f = File(ctx.filesDir, FILE)
        val recent = runCatching { f.readLines().mapNotNull { it.toLongOrNull() }.filter { now() - it in 0..WINDOW_MS } }.getOrDefault(emptyList())
        if (recent.size >= 2) {
            f.delete()
            if (Cfg.b("theme.enabled")) {
                Cfg.set("theme.enabled", "0")
                Cfg.save()
            }
        } else if (recent.isEmpty()) {
            f.delete()
        }
    }
}

/** Lleva al usuario al launcher original (Google TV) para cuando el tema está apagado. */
object DefaultHome {
    private val GOOGLE_TV = ComponentName("com.google.android.apps.tv.launcherx", "com.google.android.apps.tv.launcherx.home.HomeActivity")

    fun open(ctx: Context) {
        val base = Intent(Intent.ACTION_MAIN).addCategory(Intent.CATEGORY_HOME).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
        val tryGoogle = Intent(base).setComponent(GOOGLE_TV)
        if (runCatching { ctx.startActivity(tryGoogle) }.isSuccess) return
        // Si Google TV no está, cualquier otro inicio que no sea el nuestro.
        val other = ctx.packageManager.queryIntentActivities(base, PackageManager.MATCH_DEFAULT_ONLY)
            .map { it.activityInfo }
            .firstOrNull { it.packageName != ctx.packageName }
        if (other != null) runCatching { ctx.startActivity(Intent(base).setClassName(other.packageName, other.name)) }
    }
}

fun nowMs(): Long = SystemClock.uptimeMillis()
