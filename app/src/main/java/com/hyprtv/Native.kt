package com.hyprtv

import java.nio.ByteBuffer

/** Puente JNI con el núcleo Rust (`core/src/lib.rs`). Los tiempos son `SystemClock.uptimeMillis()`. */
object Native {
    /** Bit que `petTick` activa cuando hay un fotograma nuevo en el búfer. */
    const val DIRTY = 1 shl 30

    /** En los 16 bits bajos de `petTick`: ms hasta el siguiente fotograma (0 = no pedir más). */
    const val DELAY_MASK = 0xFFFF

    /** Los 4 bits siguientes: lo que la mascota acaba de empezar a hacer (`EV_*`). */
    const val EVENT_SHIFT = 16
    const val EVENT_MASK = 0xF

    // `decide`: de dónde viene la ocasión de hablar.
    const val TRIGGER_WINDOW = 0
    const val TRIGGER_FOCUS = 1
    const val TRIGGER_IDLE = 2
    const val TRIGGER_MANUAL = 3
    const val TRIGGER_ACTIVITY = 4

    // `keyPress`: qué atajo se completó (0 = ninguno).
    const val ACTION_PET = 1
    const val ACTION_THEME = 2
    const val ACTION_RING = 3
    const val ACTION_CORNER = 4
    const val ACTION_RING_APP = 5

    // Lo que hace la mascota en su rutina (el nombre es el que entiende `decide` en «act:<nombre>»).
    val EVENT_NAMES = arrayOf("", "bored", "walk", "dance", "point", "wave", "rest", "look", "stretch", "tired", "back")
    const val EV_TIRED = 9
    const val EV_BACK = 10

    init {
        System.loadLibrary("hyprcore")
    }

    @JvmStatic external fun loadConfig(path: String): String
    @JvmStatic external fun saveConfig(path: String, dump: String): String
    @JvmStatic external fun applyPreset(dump: String, name: String): String
    @JvmStatic external fun presetNames(): String
    @JvmStatic external fun petCreate(w: Int, h: Int, now: Long)
    @JvmStatic external fun petWorld(home: Float, lo: Float, hi: Float, now: Long)
    @JvmStatic external fun petFocus(x: Float)
    @JvmStatic external fun petRoutine(now: Long, ms: Long)
    @JvmStatic external fun petInterrupt(now: Long)
    @JvmStatic external fun petX(): Float
    @JvmStatic external fun petWake(now: Long)
    @JvmStatic external fun petTalk(now: Long, ms: Int)
    @JvmStatic external fun petTick(now: Long, buf: ByteBuffer): Int
    @JvmStatic external fun isQuiet(hour: Int): Boolean
    @JvmStatic external fun decide(trigger: Int, now: Long, hour: Int, pkg: String, text: String, title: String): String?
    @JvmStatic external fun keyPress(code: Int, now: Long): Int
}
