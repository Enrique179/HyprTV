package com.hyprtv

import android.accessibilityservice.AccessibilityService
import android.content.Context
import android.content.Intent
import android.graphics.Rect
import android.media.AudioAttributes
import android.media.AudioManager
import android.media.AudioPlaybackConfiguration
import android.os.Handler
import android.os.Looper
import android.os.PowerManager
import android.view.KeyEvent
import android.view.WindowManager
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityNodeInfo
import java.util.Calendar

/**
 * El servicio que hace de "tema de sistema": dibuja el borde de foco y la mascota sobre cualquier app, lee el título de lo
 * enfocado para los chistes y escucha los atajos del mando. Va por eventos (cambio de ventana y de foco): mientras nadie
 * navega no hace nada, salvo la rutina de la mascota cuando la TV lleva un rato sin usarse.
 */
class ScreenService : AccessibilityService() {
    companion object {
        @Volatile var instance: ScreenService? = null

        /** Ventanas que no son "la pantalla" (volumen, teclado, notificaciones). */
        private val IGNORED = setOf("com.android.systemui", "android", "com.google.android.inputmethod.latin", "com.android.inputmethod.latin")
        private val OWN_SCREENS = setOf("com.hyprtv.HomeActivity", "com.hyprtv.SettingsActivity")
        private const val IDLE_MS = 150_000L
        private const val DWELL_MS = 3_000L
        private const val WINDOW_DELAY_MS = 900L

        /** Cuántas rutinas seguidas hace sin que nadie toque nada antes de dejarlo por imposible. */
        private const val MAX_ROUTINES = 5
    }

    private val handler = Handler(Looper.getMainLooper())
    private lateinit var wm: WindowManager
    private lateinit var audio: AudioManager
    private lateinit var power: PowerManager
    private var pet: PetOverlay? = null
    private var ring: RingOverlay? = null
    private var osd: Osd? = null
    private var currentPkg = ""
    private var playing = false
    private var lowMemory = false
    private var screenW = 1280
    private var screenH = 720

    // Última vez que alguien usó el mando o cambió la pantalla, para saber cuándo "la TV no se usa".
    private var lastInput = 0L
    private var routines = 0
    private var focusTitle = ""

    private val configListener: () -> Unit = { handler.post { apply() } }
    private var contextJob: Runnable? = null
    private var dwellJob: Runnable? = null
    private val idleJob = object : Runnable {
        override fun run() {
            speak(Native.TRIGGER_IDLE, currentPkg, "", "")
            handler.postDelayed(this, IDLE_MS)
        }
    }
    private val routineJob = Runnable { maybeStartRoutine() }

    // Listas del borde por app, ya troceadas (se vuelven a leer solo si cambió la configuración).
    private var ringSetsAt = -1
    private var ringAllow = emptySet<String>()
    private var ringDeny = emptySet<String>()

    private val playbackCallback = object : AudioManager.AudioPlaybackCallback() {
        override fun onPlaybackConfigChanged(configs: MutableList<AudioPlaybackConfiguration>) {
            val media = configs.any {
                val usage = it.audioAttributes.usage
                usage == AudioAttributes.USAGE_MEDIA || usage == AudioAttributes.USAGE_GAME
            }
            handler.post {
                if (media != playing) {
                    playing = media
                    apply()
                }
            }
        }
    }

    override fun onServiceConnected() {
        instance = this
        Cfg.init(applicationContext)
        wm = getSystemService(Context.WINDOW_SERVICE) as WindowManager
        audio = getSystemService(Context.AUDIO_SERVICE) as AudioManager
        power = getSystemService(Context.POWER_SERVICE) as PowerManager
        val m = resources.displayMetrics
        screenW = m.widthPixels
        screenH = m.heightPixels
        lastInput = nowMs()
        audio.registerAudioPlaybackCallback(playbackCallback, handler)
        Cfg.onChange(configListener)
        apply()
    }

    override fun onUnbind(intent: Intent?): Boolean {
        shutdown()
        return super.onUnbind(intent)
    }

    override fun onDestroy() {
        shutdown()
        super.onDestroy()
    }

    private fun shutdown() {
        instance = null
        handler.removeCallbacksAndMessages(null)
        runCatching { audio.unregisterAudioPlaybackCallback(playbackCallback) }
        Cfg.removeOnChange(configListener)
        pet?.hide()
        ring?.hide()
        pet = null
        ring = null
    }

    override fun onInterrupt() {}

    @Suppress("DEPRECATION")
    override fun onTrimMemory(level: Int) {
        super.onTrimMemory(level)
        // Con la RAM justa lo primero que sobra es la mascota: se esconde un minuto.
        if (level >= TRIM_MEMORY_RUNNING_LOW && !lowMemory) {
            lowMemory = true
            apply()
            handler.postDelayed({ lowMemory = false; apply() }, 60_000)
        }
    }

    /** Pone las ventanas como dice la configuración. Se llama al conectar y con cada cambio. */
    private fun apply() {
        if (!Cfg.b("theme.enabled")) {
            pet?.hide()
            ring?.hide()
            handler.removeCallbacks(idleJob)
            handler.removeCallbacks(routineJob)
            return
        }
        if (Cfg.b("pet.enabled") && !playing && !lowMemory) {
            val p = pet ?: PetOverlay(this, wm, handler).also {
                pet = it
                it.onEvent = ::onPetEvent
                it.onSleep = ::onPetSleep
            }
            if (p.shown) p.syncConfig() else p.show()
        } else {
            pet?.hide()
        }
        if (!Cfg.b("ring.enabled") || Cfg.s("ring.mode") == "ninguna") ring?.hide()
        // El comentario suelto cada rato solo cuando no hay rutina: con rutina ya habla la mascota por sus actividades.
        handler.removeCallbacks(idleJob)
        if (pet?.shown == true && !Cfg.b("pet.idle")) handler.postDelayed(idleJob, IDLE_MS)
        scheduleRoutine(idleThreshold())
    }

    // ---- rutina de aburrimiento --------------------------------------------------------------

    private fun idleThreshold() = Cfg.i("pet.idle.minutes").coerceIn(1, 30) * 60_000L

    private fun scheduleRoutine(delayMs: Long) {
        handler.removeCallbacks(routineJob)
        if (Cfg.b("theme.enabled") && Cfg.b("pet.enabled") && Cfg.b("pet.idle") && pet?.shown == true) {
            handler.postDelayed(routineJob, delayMs.coerceAtLeast(1_000L))
        }
    }

    /** Alguien usó la TV: la rutina se corta y la cuenta de aburrimiento vuelve a empezar. */
    private fun onInput() {
        lastInput = nowMs()
        routines = 0
        pet?.interrupt()
        scheduleRoutine(idleThreshold())
    }

    private fun maybeStartRoutine() {
        val p = pet ?: return
        val threshold = idleThreshold()
        val idleFor = nowMs() - lastInput
        if (idleFor < threshold - 500) {
            scheduleRoutine(threshold - idleFor)
            return
        }
        val hour = Calendar.getInstance().get(Calendar.HOUR_OF_DAY)
        when {
            !p.shown || playing || lowMemory -> return // `apply()` lo vuelve a programar cuando cambie algo
            !power.isInteractive -> { scheduleRoutine(5 * 60_000L); return }
            Native.isQuiet(hour) -> { scheduleRoutine(15 * 60_000L); return }
        }
        if (routines >= MAX_ROUTINES) {
            // Ya hizo de todo: una última frase de desesperación y a dormir hasta que alguien toque algo.
            if (routines == MAX_ROUTINES) {
                routines++
                p.wake()
                handler.postDelayed({ speak(Native.TRIGGER_ACTIVITY, currentPkg, "act:abandoned", focusTitle) }, 1_500)
            }
            return
        }
        routines++
        p.startRoutine(25_000L + (Math.random() * 20_000L).toLong())
    }

    private fun onPetEvent(event: Int) {
        val kind = Native.EVENT_NAMES.getOrNull(event)?.takeIf { it.isNotEmpty() } ?: return
        speak(Native.TRIGGER_ACTIVITY, currentPkg, "act:$kind", focusTitle)
    }

    private fun onPetSleep() {
        // Se durmió: si la TV sigue sin usarse, otra rutina en un par de minutos (y si no, a la hora de siempre).
        val idleFor = nowMs() - lastInput
        val threshold = idleThreshold()
        scheduleRoutine(if (idleFor >= threshold) 90_000L + (Math.random() * 90_000L).toLong() else threshold - idleFor)
    }

    /** Para el botón «Probar la rutina» de los ajustes. */
    fun startRoutineNow() {
        val p = pet ?: PetOverlay(this, wm, handler).also {
            pet = it
            it.onEvent = ::onPetEvent
            it.onSleep = ::onPetSleep
        }
        if (!p.shown) p.show()
        p.startRoutine(30_000L)
    }

    // ---- borde de foco por app ---------------------------------------------------------------

    private fun ringSets() {
        if (ringSetsAt == Cfg.version) return
        ringSetsAt = Cfg.version
        fun split(key: String) = Cfg.s(key).split(',').map { it.trim() }.filter { it.isNotEmpty() }.toSet()
        ringAllow = split("ring.allow")
        ringDeny = split("ring.deny")
    }

    /**
     * En las apps que pintan su propio resaltado (YouTube, Xuper…) un segundo marco queda desajustado y feo, así que por
     * defecto solo se dibuja en las de la lista; el modo «todas» lo pone en cualquiera salvo en las excluidas.
     */
    private fun ringAllowed(pkg: String): Boolean {
        ringSets()
        return when (Cfg.s("ring.mode")) {
            "ninguna" -> false
            "todas" -> pkg !in ringDeny
            else -> pkg in ringAllow
        }
    }

    private fun labelOf(pkg: String): String =
        runCatching { packageManager.getApplicationLabel(packageManager.getApplicationInfo(pkg, 0)).toString() }.getOrDefault(pkg)

    /** Atajo: enciende o apaga el borde justo para la app que está delante. */
    private fun toggleRingForThisApp() {
        val pkg = activePackage()?.takeIf { it !in IGNORED && it != packageName } ?: currentPkg
        if (pkg.isEmpty() || pkg == packageName) {
            osd("borde: no hay una app delante")
            return
        }
        ringSets()
        val now = !ringAllowed(pkg)
        val list = if (Cfg.s("ring.mode") == "todas") "ring.deny" else "ring.allow"
        val set = (if (list == "ring.deny") ringDeny else ringAllow).toMutableSet()
        // En «lista» encender = añadir; en «todas» encender = quitar de las excluidas.
        if ((list == "ring.allow") == now) set.add(pkg) else set.remove(pkg)
        if (Cfg.s("ring.mode") == "ninguna") Cfg.set("ring.mode", "lista")
        Cfg.set(list, set.joinToString(","))
        Cfg.save()
        if (!now) ring?.hide()
        osd("borde en ${labelOf(pkg)}: ${if (now) "sí" else "no"}")
    }

    // ---- eventos de la pantalla --------------------------------------------------------------

    /** La app de la ventana activa. Es la fuente fiable: los eventos de "ventana nueva" a veces llegan con la app que se va. */
    private fun activePackage(): String? = rootInActiveWindow?.packageName?.toString()

    // Volver a una app que ya estaba abierta no emite "ventana nueva": se detecta por el cambio de ventana activa.
    private val windowCheck = Runnable {
        val pkg = activePackage() ?: return@Runnable
        if (pkg !in IGNORED && pkg != currentPkg) onWindow(pkg)
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent) {
        if (!Cfg.b("theme.enabled")) return
        if (event.eventType == AccessibilityEvent.TYPE_WINDOWS_CHANGED) {
            val relevant = AccessibilityEvent.WINDOWS_CHANGE_ACTIVE or AccessibilityEvent.WINDOWS_CHANGE_ADDED
            if (event.windowChanges and relevant != 0) {
                handler.removeCallbacks(windowCheck)
                handler.postDelayed(windowCheck, 400)
            }
            return
        }
        val pkg = event.packageName?.toString() ?: return
        if (pkg in IGNORED) return
        // Nuestras propias ventanas superpuestas también avisan de "ventana nueva": solo cuentan el inicio y los ajustes.
        if (pkg == packageName && event.eventType == AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED &&
            event.className?.toString() !in OWN_SCREENS
        ) return
        when (event.eventType) {
            AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED -> onWindow(pkg)
            AccessibilityEvent.TYPE_VIEW_FOCUSED -> onFocus(pkg, event.source)
        }
    }

    private fun onWindow(pkg: String) {
        currentPkg = pkg
        focusTitle = ""
        ring?.hide()
        onInput()
        pet?.wake()
        contextJob?.let { handler.removeCallbacks(it) }
        // Se espera a que la pantalla termine de pintarse antes de leerla.
        contextJob = Runnable {
            // Si entretanto la app activa es otra, este comentario ya no viene a cuento.
            val active = activePackage()
            if (active == null || active == pkg) speak(Native.TRIGGER_WINDOW, pkg, screenText(), "")
        }.also { handler.postDelayed(it, WINDOW_DELAY_MS) }
        rearmIdle()
    }

    private fun onFocus(pkg: String, node: AccessibilityNodeInfo?) {
        node ?: return
        val rect = Rect()
        node.getBoundsInScreen(rect)
        onInput()
        val tooBig = rect.width() > screenW * 0.9f && rect.height() > screenH * 0.9f
        // Solo elementos de verdad visibles y con tamaño de elemento (hay apps que enfocan vistas invisibles de unos píxeles).
        // En el inicio y los ajustes de HyprTV el foco ya tiene su propio borde: dos marcos seguidos quedarían feos.
        val minSide = Style.dp(24).toInt()
        val onScreen = rect.right > 0 && rect.bottom > 0 && rect.left < screenW && rect.top < screenH
        if (Cfg.b("ring.enabled") && pkg != packageName && ringAllowed(pkg) && node.isVisibleToUser && onScreen &&
            rect.width() >= minSide && rect.height() >= minSide && !tooBig
        ) {
            (ring ?: RingOverlay(this, wm).also { ring = it }).show(rect)
        } else {
            ring?.hide()
        }
        pet?.setFocusX(((rect.left + rect.right) / 2f / screenW).coerceIn(0f, 1f))
        pet?.avoid(rect, screenH)
        pet?.dismissBubbleIfStale()

        val title = titleOf(node)
        focusTitle = title
        dwellJob?.let { handler.removeCallbacks(it) }
        if (title.isNotEmpty()) {
            dwellJob = Runnable { speak(Native.TRIGGER_FOCUS, pkg, title, title) }.also { handler.postDelayed(it, DWELL_MS) }
        }
        rearmIdle()
    }

    private fun rearmIdle() {
        handler.removeCallbacks(idleJob)
        if (pet?.shown == true && !Cfg.b("pet.idle")) handler.postDelayed(idleJob, IDLE_MS)
    }

    /** Primer texto corto y visible del elemento enfocado (en las tarjetas de TV el título está en un hijo). */
    private fun titleOf(node: AccessibilityNodeInfo): String {
        val out = StringBuilder()
        collect(node, out, intArrayOf(12), 0, stopAtFirst = true)
        return out.toString().trim()
    }

    private fun screenText(): String {
        val root = rootInActiveWindow ?: return ""
        val out = StringBuilder()
        collect(root, out, intArrayOf(70), 0, stopAtFirst = false)
        return out.toString()
    }

    private fun collect(node: AccessibilityNodeInfo?, out: StringBuilder, budget: IntArray, depth: Int, stopAtFirst: Boolean) {
        if (node == null || budget[0] <= 0 || depth > 9 || (stopAtFirst && out.isNotEmpty())) return
        budget[0]--
        if (node.isVisibleToUser) {
            val t = node.text ?: node.contentDescription
            if (t != null && t.length in 2..80 && out.length < 700) out.append(t).append(' ')
        }
        for (i in 0 until node.childCount) collect(node.getChild(i), out, budget, depth + 1, stopAtFirst)
    }

    // ---- hablar ------------------------------------------------------------------------------

    private fun speak(trigger: Int, pkg: String, text: String, title: String) {
        if (pet?.shown != true) return
        val hour = Calendar.getInstance().get(Calendar.HOUR_OF_DAY)
        val line = Native.decide(trigger, nowMs(), hour, pkg, text, title) ?: return
        say(line)
    }

    /** Dice `line` ahora (también lo usa el botón "Probar un chiste" de los ajustes). */
    fun say(line: String) {
        (pet ?: PetOverlay(this, wm, handler).also {
            pet = it
            it.onEvent = ::onPetEvent
            it.onSleep = ::onPetSleep
        }).let {
            if (!it.shown) it.show()
            it.say(line, (3500L + line.length * 55L).coerceAtMost(9000L))
        }
    }

    // ---- atajos del mando --------------------------------------------------------------------

    // La TV (Google TV + Realtek) abre siempre su launcher con la tecla Home, aunque HyprTV sea el inicio elegido: se
    // intercepta aquí. Si este servicio no está activo, Home sigue abriendo Google TV y nada se rompe.
    private var homeConsumed = false

    private fun openOwnHome(): Boolean {
        val i = Intent(this, HomeActivity::class.java).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP)
        return runCatching { startActivity(i) }.isSuccess
    }

    override fun onKeyEvent(event: KeyEvent): Boolean {
        if (event.action == KeyEvent.ACTION_DOWN && event.repeatCount == 0 && Cfg.b("theme.enabled")) onInput()
        if (event.keyCode == KeyEvent.KEYCODE_HOME && Cfg.b("theme.enabled")) {
            if (event.action == KeyEvent.ACTION_DOWN && event.repeatCount == 0) homeConsumed = openOwnHome()
            val consumed = homeConsumed
            if (event.action == KeyEvent.ACTION_UP) homeConsumed = false
            return consumed
        }
        if (event.action != KeyEvent.ACTION_DOWN || event.repeatCount != 0) return false
        val action = Native.keyPress(event.keyCode, nowMs())
        if (action == 0) return false
        val themeOn = Cfg.b("theme.enabled")
        // Con el tema apagado solo sigue vivo el atajo que lo vuelve a encender.
        if (!themeOn && action != Native.ACTION_THEME) return false
        when (action) {
            Native.ACTION_PET -> flip("pet.enabled", "mascota")
            Native.ACTION_RING -> flip("ring.enabled", "borde de foco")
            Native.ACTION_RING_APP -> toggleRingForThisApp()
            Native.ACTION_CORNER -> {
                Cfg.set("pet.corner", if (Cfg.s("pet.corner") == "br") "bl" else "br")
                Cfg.save()
                osd("mascota: esquina ${if (Cfg.s("pet.corner") == "br") "derecha" else "izquierda"}")
            }
            Native.ACTION_THEME -> {
                flip("theme.enabled", "tema")
                // Apagado: el inicio del sistema (Google TV). Encendido: el nuestro.
                handler.postDelayed({ if (Cfg.b("theme.enabled")) openOwnHome() else performGlobalAction(GLOBAL_ACTION_HOME) }, 250)
            }
        }
        return true
    }

    private fun flip(key: String, name: String) {
        val on = !Cfg.b(key)
        Cfg.set(key, if (on) "1" else "0")
        Cfg.save()
        osd("$name: ${if (on) "activado" else "apagado"}")
    }

    private fun osd(text: String) {
        (osd ?: Osd(this, wm, handler).also { osd = it }).show(text)
    }
}
