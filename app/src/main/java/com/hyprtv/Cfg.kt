package com.hyprtv

import android.content.Context
import android.graphics.Color
import java.io.File

/**
 * La configuración, en memoria. El archivo lo lee, valida y guarda el núcleo Rust (`config.rs`); aquí solo se guarda una
 * copia para consultarla rápido al dibujar. Si se sube un `config.ini` a `Android/data/com.hyprtv/files/` se importa al abrir.
 */
object Cfg {
    private val map = HashMap<String, String>()
    private val listeners = ArrayList<() -> Unit>()
    private var file: File? = null
    private var exported: File? = null

    /** Sube con cada cambio: las pantallas lo comparan para saber si tienen que redibujarse. */
    @Volatile var version = 0
        private set

    @Synchronized
    fun init(ctx: Context) {
        if (file != null) return
        val f = File(ctx.filesDir, "config.ini")
        val ext = ctx.getExternalFilesDir(null)?.let { File(it, "config.ini") }
        file = f
        exported = ext
        importExternal()
        parse(Native.loadConfig(f.path))
    }

    /** Vuelve a leer lo que haya en la carpeta externa (para cuando se edita desde el PC con la app abierta). */
    @Synchronized
    fun reload() {
        val f = file ?: return
        importExternal()
        parse(Native.loadConfig(f.path))
        notifyChanged()
    }

    private fun importExternal() {
        val f = file ?: return
        val e = exported ?: return
        if (e.isFile && (!f.exists() || e.lastModified() > f.lastModified())) {
            runCatching { e.copyTo(f, overwrite = true) }
        }
    }

    private fun parse(dump: String) {
        map.clear()
        for (line in dump.lineSequence()) {
            val i = line.indexOf('=')
            if (i > 0) map[line.substring(0, i)] = line.substring(i + 1)
        }
        version++
    }

    private fun dump() = map.entries.joinToString("\n") { "${it.key}=${it.value}" }

    fun s(key: String): String = map[key] ?: ""
    fun i(key: String): Int = map[key]?.toIntOrNull() ?: 0
    fun b(key: String): Boolean = map[key] == "1"

    fun color(key: String): Int = try {
        Color.parseColor(map[key])
    } catch (_: Exception) {
        Color.MAGENTA
    }

    /** Espacio (1..6) asignado a un paquete por el usuario, o `null` si no eligió ninguno. */
    fun workspaceOf(pkg: String): Int? = map["app.$pkg"]?.toIntOrNull()

    @Synchronized
    fun set(key: String, value: String) {
        map[key] = value
    }

    @Synchronized
    fun setAll(pairs: Map<String, String>) {
        map.putAll(pairs)
    }

    /** Guarda lo cambiado: el núcleo valida y normaliza, y se vuelve a leer lo que dejó. */
    @Synchronized
    fun save() {
        val f = file ?: return
        parse(Native.saveConfig(f.path, dump()))
        runCatching { exported?.let { File(f.path).copyTo(it, overwrite = true) } }
        notifyChanged()
    }

    @Synchronized
    fun preset(name: String) {
        parse(Native.applyPreset(dump(), name))
        save()
    }

    @Synchronized
    fun resetToDefaults() {
        map.clear()
        save()
    }

    fun onChange(l: () -> Unit) {
        listeners.add(l)
    }

    fun removeOnChange(l: () -> Unit) {
        listeners.remove(l)
    }

    private fun notifyChanged() {
        listeners.toList().forEach { it() }
    }
}
