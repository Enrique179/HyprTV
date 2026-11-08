package com.hyprtv

import android.app.Activity
import android.app.AlertDialog
import android.content.Context
import android.content.Intent
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.RectF
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.provider.Settings
import android.view.Gravity
import android.view.KeyEvent
import android.view.View
import android.view.ViewGroup
import android.widget.FrameLayout
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import android.widget.Toast
import java.util.Calendar

/** Una fila del panel. `header` es un título de sección sin foco. */
class Row(
    val title: String,
    val value: () -> String = { "" },
    val header: Boolean = false,
    val left: (() -> Unit)? = null,
    val right: (() -> Unit)? = null,
    val click: (() -> Unit)? = null,
)

/** Panel de personalización, todo con el mando: ◄ ► cambian el valor, OK acciona, Atrás vuelve. */
class SettingsActivity : Activity() {
    private lateinit var list: LinearLayout
    private val ui = Handler(Looper.getMainLooper())
    private var resetArmed = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val root = FrameLayout(this)
        root.addView(Background(this), ViewGroup.LayoutParams(-1, -1))

        val column = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL }
        val side = Style.dp(48).toInt()
        column.setPadding(side, Style.dp(24).toInt(), side, Style.dp(16).toInt())
        root.addView(column, FrameLayout.LayoutParams(-1, -1))

        column.addView(TextView(this).apply {
            text = "hyprtv · ajustes"
            setTextColor(Cfg.color("color.a1"))
            typeface = Style.typeface(bold = true)
            textSize = 26f
            setPadding(0, 0, 0, Style.dp(8).toInt())
        })
        list = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(0, 0, 0, Style.dp(24).toInt())
        }
        column.addView(ScrollView(this).apply {
            isVerticalScrollBarEnabled = false
            overScrollMode = View.OVER_SCROLL_NEVER
            clipChildren = false
            addView(list)
        }, LinearLayout.LayoutParams(-1, 0, 1f))
        column.addView(TextView(this).apply {
            text = "◄ ► cambiar   OK acción   Atrás volver"
            setTextColor(Cfg.color("color.muted"))
            typeface = Style.typeface()
            textSize = 15f
            setPadding(0, Style.dp(6).toInt(), 0, 0)
        })
        setContentView(root)
        render()
    }

    // ---- constructores de filas -------------------------------------------------------------

    private fun changed() {
        Cfg.save()
        render()
    }

    private fun toggle(title: String, key: String) = Row(
        title, { if (Cfg.b(key)) "sí" else "no" },
        left = { Cfg.set(key, if (Cfg.b(key)) "0" else "1"); changed() },
        right = { Cfg.set(key, if (Cfg.b(key)) "0" else "1"); changed() },
        click = { Cfg.set(key, if (Cfg.b(key)) "0" else "1"); changed() },
    )

    private fun step(title: String, key: String, step: Int, min: Int, max: Int, suffix: String = "", label: (Int) -> String = { "$it$suffix" }): Row {
        fun move(d: Int) {
            Cfg.set(key, (Cfg.i(key) + d * step).coerceIn(min, max).toString())
            changed()
        }
        return Row(title, { label(Cfg.i(key)) }, left = { move(-1) }, right = { move(1) }, click = { move(1) })
    }

    private fun cycle(title: String, key: String, options: List<String>, names: Map<String, String> = emptyMap()): Row {
        fun move(d: Int) {
            val i = options.indexOf(Cfg.s(key)).coerceAtLeast(0)
            Cfg.set(key, options[(i + d + options.size) % options.size])
            changed()
        }
        return Row(title, { names[Cfg.s(key)] ?: Cfg.s(key) }, left = { move(-1) }, right = { move(1) }, click = { move(1) })
    }

    private fun palette(): Row {
        val names = Native.presetNames().split(',') + "custom"
        fun move(d: Int) {
            val i = names.indexOf(Cfg.s("theme.preset")).coerceAtLeast(0)
            val next = names[(i + d + names.size) % names.size]
            if (next == "custom") { Cfg.set("theme.preset", "custom"); changed() } else { Cfg.preset(next); render() }
        }
        return Row("Paleta", { Cfg.s("theme.preset") }, left = { move(-1) }, right = { move(1) }, click = { move(1) })
    }

    private fun comboRow(title: String, key: String) = Row(
        title,
        { Cfg.s(key).split(',').filter { it.isNotBlank() }.joinToString(" ") { keyName(it.toInt()) }.ifEmpty { "sin asignar" } },
        left = { Cfg.set(key, ""); changed() },
        click = { learn(title, key) },
    )

    private fun action(title: String, value: String = "", run: () -> Unit) = Row(title, { value }, click = run)

    // ---- contenido ---------------------------------------------------------------------------

    private fun serviceOn(): Boolean {
        val enabled = Settings.Secure.getString(contentResolver, Settings.Secure.ENABLED_ACCESSIBILITY_SERVICES) ?: return false
        return enabled.split(':').any { it.startsWith("$packageName/") }
    }

    private fun rows(): List<Row> = listOf(
        Row("Tema", header = true),
        toggle("Tema activo (apagado = launcher de Google TV)", "theme.enabled"),
        palette(),
        step("Hueco entre mosaicos", "gap", 2, 0, 40),
        step("Esquinas redondeadas", "radius", 2, 0, 40),
        step("Grosor del borde", "border.width", 1, 1, 8),
        cycle("Estilo del borde", "border.style", listOf("gradient", "solid", "glow"), mapOf("gradient" to "degradado", "solid" to "sólido", "glow" to "resplandor")),
        toggle("Borde animado (gira)", "border.anim"),
        cycle("Fuente", "font", listOf("mono", "sans", "serif"), mapOf("mono" to "monoespaciada", "sans" to "sans", "serif" to "serif")),
        cycle("Fondo", "bg.mode", listOf("gradient", "solid", "image"), mapOf("gradient" to "degradado", "solid" to "liso", "image" to "imagen (bg.png)")),
        cycle("Diseño", "layout", listOf("grid", "master"), mapOf("grid" to "rejilla", "master" to "principal 2x2")),
        step("Columnas", "columns", 1, 3, 8),
        step("Alto del mosaico", "tile.ratio", 4, 40, 100, "%"),
        toggle("Etiquetas de los mosaicos", "tile.labels"),
        toggle("Barra de espacios", "bar.enabled"),
        toggle("Reloj de 24 horas", "clock.24h"),
        step("Número de espacios", "ws.count", 1, 1, 6),
        step("Velocidad de animaciones", "anim.speed", 25, 0, 300, "%", { if (it == 0) "sin animaciones" else "$it%" }),

        Row("Borde de foco sobre otras apps", header = true),
        toggle("Borde de foco global", "ring.enabled"),
        cycle(
            "En qué apps", "ring.mode", listOf("lista", "todas", "ninguna"),
            mapOf("lista" to "solo las elegidas", "todas" to "todas menos las excluidas", "ninguna" to "ninguna"),
        ),
        action("Elegir apps con borde…", "OK") { pickRingApps() },
        step("Separación del borde", "ring.padding", 1, 0, 20),

        Row("Mascota", header = true),
        toggle("Mascota activa", "pet.enabled"),
        toggle("Hace cosas cuando la TV no se usa", "pet.idle"),
        step("Espera antes de aburrirse", "pet.idle.minutes", 1, 1, 30, " min"),
        action("Probar la rutina ahora", "OK") { testRoutine() },
        cycle("Modelo", "pet.look", listOf("bufanda", "gorro", "coletas", "gafas", "slayer")),
        step("Tamaño", "pet.size", 10, 60, 200, "%"),
        cycle("Esquina", "pet.corner", listOf("br", "bl"), mapOf("br" to "abajo a la derecha", "bl" to "abajo a la izquierda")),
        step("Charla", "pet.chatter", 1, 0, 3, label = { listOf("callada", "poco", "normal", "charlatana")[it] }),
        toggle("Silencio nocturno", "pet.quiet"),
        step("Silencio desde las", "pet.quiet.start", 1, 0, 23, " h"),
        step("Silencio hasta las", "pet.quiet.end", 1, 0, 23, " h"),
        action("Probar un chiste", "OK") { testJoke() },

        Row("Atajos del mando (◄ borra, OK aprende)", header = true),
        comboRow("Apagar / encender la mascota", "combo.pet"),
        comboRow("Apagar / encender el tema", "combo.theme"),
        comboRow("Apagar / encender el borde global", "combo.ring"),
        comboRow("Cambiar la mascota de esquina", "combo.corner"),
        comboRow("Borde sí/no en la app de delante", "combo.ringapp"),

        Row("Sistema", header = true),
        action("Servicio de accesibilidad", if (serviceOn()) "activo" else "apagado (OK para activarlo)") {
            startActivity(Intent(Settings.ACTION_ACCESSIBILITY_SETTINGS))
        },
        action("Ajustes de Android", "OK") { startActivity(Intent(Settings.ACTION_SETTINGS)) },
        action("Ir a Google TV ahora", "OK") { DefaultHome.open(this) },
        action("Recargar config.ini del PC", "OK") { Cfg.reload(); render(); toast("Configuración recargada") },
        action("Restaurar valores por defecto", if (resetArmed) "¡OK otra vez para confirmar!" else "OK") {
            if (resetArmed) { resetArmed = false; Cfg.resetToDefaults(); render(); toast("Valores restaurados") } else { resetArmed = true; render() }
        },
    )

    private fun render() {
        val focusedIndex = (0 until list.childCount).firstOrNull { list.getChildAt(it).hasFocus() } ?: -1
        list.removeAllViews()
        var firstFocusable: View? = null
        for (row in rows()) {
            val v = RowView(this, row)
            if (!row.header && firstFocusable == null) firstFocusable = v
            list.addView(v, LinearLayout.LayoutParams(-1, -2).apply {
                topMargin = if (row.header) Style.dp(14).toInt() else Style.dp(Cfg.i("gap").coerceIn(2, 12)).toInt()
            })
        }
        (list.getChildAt(focusedIndex.coerceAtLeast(0)).takeIf { focusedIndex >= 0 && it.isFocusable } ?: firstFocusable)?.requestFocus()
    }

    private fun toast(s: String) = Toast.makeText(this, s, Toast.LENGTH_SHORT).show()

    private fun testJoke() {
        val hour = Calendar.getInstance().get(Calendar.HOUR_OF_DAY)
        val line = Native.decide(Native.TRIGGER_MANUAL, nowMs(), hour, packageName, "", "")
        val service = ScreenService.instance
        when {
            line == null -> toast("Sin chiste por ahora")
            service != null -> { service.say(line); toast("Mira la esquina de la pantalla…") }
            else -> toast(line)
        }
    }

    private fun testRoutine() {
        val service = ScreenService.instance
        if (service == null) {
            toast("Activa el servicio de accesibilidad primero")
            return
        }
        service.startRoutineNow()
        toast("Pulsa cualquier tecla para pararla")
    }

    /** Lista de apps con casilla: qué apps reciben el borde (en modo «todas», cuáles se excluyen). */
    private fun pickRingApps() {
        val all = Apps.load(this).filter { !it.special }.toMutableList()
        // Ajustes de Android no sale en la lista de apps del launcher, y es justo donde mejor queda el borde.
        if (all.none { it.pkg == "com.android.tv.settings" }) {
            all.add(0, AppEntry("com.android.tv.settings", "Ajustes de Android", Intent(), 3))
        }
        val inclusive = Cfg.s("ring.mode") != "todas"
        val key = if (inclusive) "ring.allow" else "ring.deny"
        val listed = Cfg.s(key).split(',').map { it.trim() }.filter { it.isNotEmpty() }.toMutableSet()
        val checked = BooleanArray(all.size) { (all[it].pkg in listed) == inclusive }
        AlertDialog.Builder(this, AlertDialog.THEME_DEVICE_DEFAULT_DARK)
            .setTitle(if (inclusive) "Apps con borde de foco" else "Apps SIN borde de foco")
            .setMultiChoiceItems(all.map { it.label }.toTypedArray(), checked) { _, i, on ->
                checked[i] = on
            }
            .setPositiveButton("Guardar") { _, _ ->
                for (i in all.indices) {
                    val pkg = all[i].pkg
                    // En «lista» marcar = añadir; en «todas» marcar = que tenga borde = quitar de las excluidas.
                    if (checked[i] == inclusive) listed.add(pkg) else listed.remove(pkg)
                }
                Cfg.set(key, listed.joinToString(","))
                changed()
            }
            .setNegativeButton("Cancelar", null)
            .show()
    }

    // ---- aprender atajos ---------------------------------------------------------------------

    private fun keyName(code: Int) = KeyEvent.keyCodeToString(code).removePrefix("KEYCODE_").lowercase()

    private fun learn(title: String, key: String) {
        val seq = ArrayList<Int>()
        val dialog = AlertDialog.Builder(this, AlertDialog.THEME_DEVICE_DEFAULT_DARK).setTitle(title).create()
        fun text() = "Pulsa la secuencia de teclas del mando que quieras (de 2 a 6 pulsaciones) y espera un momento.\n" +
            "No valen las flechas, OK ni Atrás.\n\nTeclas: " + seq.joinToString(" ") { keyName(it) }
        dialog.setMessage(text())
        val finish = Runnable {
            if (seq.size >= 2) {
                Cfg.set(key, seq.joinToString(","))
                changed()
            } else {
                toast("Hacen falta al menos 2 pulsaciones")
            }
            dialog.dismiss()
        }
        val forbidden = setOf(
            KeyEvent.KEYCODE_DPAD_UP, KeyEvent.KEYCODE_DPAD_DOWN, KeyEvent.KEYCODE_DPAD_LEFT, KeyEvent.KEYCODE_DPAD_RIGHT,
            KeyEvent.KEYCODE_DPAD_CENTER, KeyEvent.KEYCODE_ENTER, KeyEvent.KEYCODE_HOME,
        )
        dialog.setOnKeyListener { _, code, ev ->
            if (code == KeyEvent.KEYCODE_BACK) {
                if (ev.action == KeyEvent.ACTION_UP) { ui.removeCallbacks(finish); dialog.dismiss() }
            } else if (ev.action == KeyEvent.ACTION_DOWN && ev.repeatCount == 0 && code !in forbidden && seq.size < 6) {
                seq.add(code)
                dialog.setMessage(text())
                ui.removeCallbacks(finish)
                ui.postDelayed(finish, 1500)
            }
            true
        }
        dialog.show()
    }
}

/** Fila del panel dibujada con el tema: título a la izquierda, valor a la derecha y ◄ ► cuando tiene el foco. */
class RowView(ctx: Context, private val row: Row) : View(ctx) {
    private val fill = Paint(Paint.ANTI_ALIAS_FLAG)
    private val border = Paint(Paint.ANTI_ALIAS_FLAG)
    private val text = Paint(Paint.ANTI_ALIAS_FLAG)
    private val rect = RectF()

    init {
        isFocusable = !row.header
        isClickable = !row.header
        defaultFocusHighlightEnabled = false
        row.click?.let { c -> setOnClickListener { c() } }
    }

    override fun onMeasure(w: Int, h: Int) {
        val hh = if (row.header) Style.dp(30) else Style.dp(44)
        setMeasuredDimension(MeasureSpec.getSize(w), hh.toInt())
    }

    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean {
        when (keyCode) {
            KeyEvent.KEYCODE_DPAD_LEFT -> if (row.left != null) { row.left.invoke(); return true }
            KeyEvent.KEYCODE_DPAD_RIGHT -> if (row.right != null) { row.right.invoke(); return true }
        }
        return super.onKeyDown(keyCode, event)
    }

    override fun onFocusChanged(gainFocus: Boolean, direction: Int, previouslyFocusedRect: android.graphics.Rect?) {
        super.onFocusChanged(gainFocus, direction, previouslyFocusedRect)
        invalidate()
    }

    override fun onDraw(c: Canvas) {
        text.typeface = Style.typeface(bold = row.header)
        if (row.header) {
            text.textSize = Style.sp(15f)
            text.color = Cfg.color("color.a3")
            c.drawText(row.title.uppercase(), Style.dp(4), height - Style.dp(6), text)
            return
        }
        val r = Style.dp(Cfg.i("radius").coerceAtMost(14).toFloat())
        val bw = Style.dp(Cfg.i("border.width").coerceAtMost(3).toFloat())
        rect.set(bw / 2, bw / 2, width - bw / 2, height - bw / 2)
        fill.color = Cfg.color("color.panel")
        c.drawRoundRect(rect, r, r, fill)
        border.style = Paint.Style.STROKE
        border.strokeWidth = Style.dp(1)
        border.color = Cfg.color("color.border")
        c.drawRoundRect(rect, r, r, border)
        if (isFocused) Style.activeBorder(c, rect, r, bw, border)

        text.textSize = Style.sp(17f)
        text.color = Cfg.color("color.text")
        val base = height / 2f - (text.fontMetrics.ascent + text.fontMetrics.descent) / 2f
        c.drawText(row.title, Style.dp(16), base, text)

        var value = row.value()
        if (isFocused && (row.left != null || row.right != null) && value.isNotEmpty()) value = "◄ $value ►"
        text.color = if (isFocused) Cfg.color("color.a1") else Cfg.color("color.muted")
        c.drawText(value, width - Style.dp(16) - text.measureText(value), base, text)
    }
}
