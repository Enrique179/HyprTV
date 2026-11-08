package com.hyprtv

import android.app.Activity
import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.BitmapShader
import android.graphics.Canvas
import android.graphics.LinearGradient
import android.graphics.Matrix
import android.graphics.Paint
import android.graphics.Path
import android.graphics.RadialGradient
import android.graphics.RectF
import android.graphics.Shader
import android.os.Bundle
import android.text.TextUtils
import android.view.Gravity
import android.view.KeyEvent
import android.view.View
import android.view.ViewGroup
import android.view.WindowManager
import android.widget.FrameLayout
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextClock
import android.widget.Toast
import java.io.File

/** El inicio: mosaicos con huecos y bordes degradados, espacios numerados arriba como una waybar. */
class HomeActivity : Activity() {
    private lateinit var icons: IconLoader
    private lateinit var grid: Grid
    private lateinit var scroll: ScrollView
    private var pills = ArrayList<Pill>()
    private var apps: List<AppEntry> = emptyList()
    private var appsKey = ""
    private var ws = 1
    private var builtWith = -1

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        if (!Cfg.b("theme.enabled")) {
            leaveForDefault()
            return
        }
        window.addFlags(WindowManager.LayoutParams.FLAG_LAYOUT_NO_LIMITS)
        icons = IconLoader(applicationContext)
        build()
    }

    override fun onResume() {
        super.onResume()
        if (!Cfg.b("theme.enabled")) {
            leaveForDefault()
            return
        }
        if (!::icons.isInitialized) return
        // Cambió la configuración o la lista de apps mientras estábamos en otra pantalla.
        val key = Apps.load(this).joinToString("|") { it.pkg + (Cfg.workspaceOf(it.pkg) ?: 0) }
        if (builtWith != Cfg.version || key != appsKey) {
            icons.clear()
            build()
        }
    }

    override fun onTrimMemory(level: Int) {
        super.onTrimMemory(level)
        if (::icons.isInitialized && level >= TRIM_MEMORY_BACKGROUND) icons.clear()
    }

    private fun leaveForDefault() {
        DefaultHome.open(this)
        finish()
    }

    // El botón Atrás en el inicio no hace nada.
    @Deprecated("Deprecated in Java")
    override fun onBackPressed() {}

    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean {
        val count = Cfg.i("ws.count")
        when (keyCode) {
            in KeyEvent.KEYCODE_1..KeyEvent.KEYCODE_6 -> {
                val n = keyCode - KeyEvent.KEYCODE_0
                if (n <= count) { showWorkspace(n); return true }
            }
            KeyEvent.KEYCODE_CHANNEL_UP, KeyEvent.KEYCODE_PAGE_UP -> { showWorkspace(if (ws <= 1) count else ws - 1); return true }
            KeyEvent.KEYCODE_CHANNEL_DOWN, KeyEvent.KEYCODE_PAGE_DOWN -> { showWorkspace(if (ws >= count) 1 else ws + 1); return true }
        }
        return super.onKeyDown(keyCode, event)
    }

    private fun build() {
        builtWith = Cfg.version
        apps = Apps.load(this)
        appsKey = apps.joinToString("|") { it.pkg + (Cfg.workspaceOf(it.pkg) ?: 0) }
        val count = Cfg.i("ws.count")
        if (ws > count) ws = 1

        val root = FrameLayout(this)
        root.addView(Background(this), ViewGroup.LayoutParams(-1, -1))

        val column = LinearLayout(this).apply { orientation = LinearLayout.VERTICAL }
        // Margen de seguridad de las teles (sobrebarrido): ~5 % a los lados y ~4 % arriba y abajo.
        val side = Style.dp(48).toInt()
        val vert = Style.dp(24).toInt()
        column.setPadding(side, vert, side, vert)
        root.addView(column, FrameLayout.LayoutParams(-1, -1))

        pills = ArrayList()
        if (Cfg.b("bar.enabled")) column.addView(makeBar(count), LinearLayout.LayoutParams(-1, -2))

        grid = Grid(this)
        scroll = ScrollView(this).apply {
            isVerticalScrollBarEnabled = false
            overScrollMode = View.OVER_SCROLL_NEVER
            clipToPadding = false
            clipChildren = false
            addView(grid, FrameLayout.LayoutParams(-1, -2))
        }
        column.addView(scroll, LinearLayout.LayoutParams(-1, 0, 1f))

        setContentView(root)
        showWorkspace(ws, focusFirst = true)
    }

    private fun makeBar(count: Int): View {
        val bar = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER_VERTICAL
            setPadding(0, 0, 0, Style.dp(12).toInt())
        }
        for (n in 1..count) {
            val pill = Pill(this, n, Cfg.s("ws.$n.name")) { showWorkspace(it) }
            pills.add(pill)
            bar.addView(pill, LinearLayout.LayoutParams(-2, -2).apply { rightMargin = Style.dp(Cfg.i("gap")).toInt() })
        }
        bar.addView(View(this), LinearLayout.LayoutParams(0, 1, 1f))
        val clock = TextClock(this).apply {
            format24Hour = "EEE d MMM  HH:mm"
            format12Hour = "EEE d MMM  h:mm a"
            if (!Cfg.b("clock.24h")) format24Hour = format12Hour
            setTextColor(Cfg.color("color.muted"))
            typeface = Style.typeface()
            textSize = 18f
        }
        bar.addView(clock, LinearLayout.LayoutParams(-2, -2))
        return bar
    }

    private fun showWorkspace(n: Int, focusFirst: Boolean = false) {
        ws = n
        // Si el foco está en una píldora (se navega por los espacios) se queda ahí; si estaba en un mosaico, se pierde con
        // la reconstrucción y hay que dárselo al primero del espacio nuevo.
        val focusOnPill = pills.any { it.isFocused }
        pills.forEach { it.active = it.ws == n; it.invalidate() }
        grid.removeAllViews()
        for (app in apps) {
            if (app.workspace.coerceAtMost(Cfg.i("ws.count")) != n) continue
            grid.addView(
                Tile(this, app, icons, ::launch, ::moveToNextWorkspace),
                ViewGroup.LayoutParams(-2, -2),
            )
        }
        val fade = Style.animMs(140)
        if (fade > 0) {
            grid.alpha = 0f
            grid.animate().alpha(1f).setDuration(fade).start()
        }
        scroll.scrollTo(0, 0)
        if (focusFirst || !focusOnPill) grid.getChildAt(0)?.requestFocus()
    }

    private fun launch(app: AppEntry) {
        runCatching { startActivity(app.intent) }.onFailure {
            Toast.makeText(this, "No se pudo abrir ${app.label}", Toast.LENGTH_SHORT).show()
        }
    }

    /** Pulsación larga: pasa la app al siguiente espacio (1→2→3→…→1). */
    private fun moveToNextWorkspace(app: AppEntry) {
        val count = Cfg.i("ws.count")
        val next = if (app.workspace >= count) 1 else app.workspace + 1
        Cfg.set("app.${app.pkg}", next.toString())
        Cfg.save()
        appsKey = ""
        Toast.makeText(this, "${app.label} → $next:${Cfg.s("ws.$next.name")}", Toast.LENGTH_SHORT).show()
        build()
    }
}

/** El fondo: degradado, color liso o una imagen propia, con un resplandor muy suave arriba a la derecha. */
class Background(ctx: Context) : View(ctx) {
    private val paint = Paint(Paint.ANTI_ALIAS_FLAG)
    private var image: Bitmap? = null
    private var shader: Shader? = null

    init {
        if (Cfg.s("bg.mode") == "image") {
            val f = File(ctx.getExternalFilesDir(null), Cfg.s("bg.image").substringAfterLast('/'))
            if (f.isFile) {
                // RGB_565 y reducida: un fondo no necesita más y así pesa la mitad.
                val o = BitmapFactory.Options().apply { inPreferredConfig = Bitmap.Config.RGB_565; inSampleSize = 2 }
                image = runCatching { BitmapFactory.decodeFile(f.path, o) }.getOrNull()
            }
        }
    }

    override fun onSizeChanged(w: Int, h: Int, ow: Int, oh: Int) {
        shader = when (Cfg.s("bg.mode")) {
            "solid" -> null
            else -> LinearGradient(0f, 0f, w.toFloat(), h.toFloat(), Cfg.color("color.bg"), Cfg.color("color.bg2"), Shader.TileMode.CLAMP)
        }
    }

    override fun onDraw(c: Canvas) {
        val img = image
        if (img != null) {
            val m = Matrix().apply {
                val s = maxOf(width / img.width.toFloat(), height / img.height.toFloat())
                setScale(s, s)
                postTranslate((width - img.width * s) / 2f, (height - img.height * s) / 2f)
            }
            c.drawBitmap(img, m, null)
            // Un velo oscuro para que las etiquetas se lean sobre cualquier foto.
            paint.shader = null
            paint.color = Style.withAlpha(Cfg.color("color.bg"), 0.55f)
            c.drawRect(0f, 0f, width.toFloat(), height.toFloat(), paint)
            return
        }
        paint.shader = shader
        paint.color = Cfg.color("color.bg")
        c.drawRect(0f, 0f, width.toFloat(), height.toFloat(), paint)
        // Resplandor: da profundidad sin necesitar desenfoque (que en esta GPU saldría caro).
        val g = RadialGradient(width * 0.85f, height * 0.05f, width * 0.6f, Style.withAlpha(Cfg.color("color.a2"), 0.16f), 0, Shader.TileMode.CLAMP)
        paint.shader = g
        c.drawRect(0f, 0f, width.toFloat(), height.toFloat(), paint)
        paint.shader = null
    }
}

/** Una píldora de espacio de trabajo en la barra: `1:apps`. */
class Pill(ctx: Context, val ws: Int, private val name: String, private val onSelect: (Int) -> Unit) : View(ctx) {
    var active = false
    private val text = Paint(Paint.ANTI_ALIAS_FLAG)
    private val fill = Paint(Paint.ANTI_ALIAS_FLAG)
    private val border = Paint(Paint.ANTI_ALIAS_FLAG)
    private val label = "$ws:$name"
    private val rect = RectF()

    init {
        isFocusable = true
        isClickable = true
        defaultFocusHighlightEnabled = false
        setOnClickListener { onSelect(ws) }
    }

    override fun onFocusChanged(gainFocus: Boolean, direction: Int, previouslyFocusedRect: android.graphics.Rect?) {
        super.onFocusChanged(gainFocus, direction, previouslyFocusedRect)
        // Pasar el foco por las píldoras cambia de espacio al momento, como en Hyprland.
        if (gainFocus) onSelect(ws)
        invalidate()
    }

    override fun onMeasure(w: Int, h: Int) {
        text.typeface = Style.typeface(bold = true)
        text.textSize = Style.sp(17f)
        val padX = Style.dp(16).toInt()
        val padY = Style.dp(8).toInt()
        setMeasuredDimension(text.measureText(label).toInt() + padX * 2, (text.fontMetrics.descent - text.fontMetrics.ascent).toInt() + padY * 2)
    }

    override fun onDraw(c: Canvas) {
        val r = Style.dp(Cfg.i("radius").coerceAtMost(14).toFloat())
        val bw = Style.dp(Cfg.i("border.width").coerceAtMost(3).toFloat())
        rect.set(bw / 2, bw / 2, width - bw / 2, height - bw / 2)
        if (active) {
            fill.shader = Style.gradient(rect)
            c.drawRoundRect(rect, r, r, fill)
            fill.shader = null
            text.color = Cfg.color("color.bg")
        } else {
            fill.color = Cfg.color("color.panel")
            c.drawRoundRect(rect, r, r, fill)
            border.style = Paint.Style.STROKE
            border.strokeWidth = Style.dp(1)
            border.color = Cfg.color("color.border")
            c.drawRoundRect(rect, r, r, border)
            text.color = Cfg.color("color.muted")
        }
        if (isFocused) Style.activeBorder(c, rect, r, bw, border)
        val y = height / 2f - (text.fontMetrics.ascent + text.fontMetrics.descent) / 2f
        c.drawText(label, (width - text.measureText(label)) / 2f, y, text)
    }
}

/** Un mosaico de app: banner o icono, etiqueta en monoespaciada y el borde activo cuando tiene el foco. */
class Tile(
    ctx: Context,
    private val entry: AppEntry,
    private val icons: IconLoader,
    onLaunch: (AppEntry) -> Unit,
    onMove: (AppEntry) -> Unit,
) : View(ctx) {
    private var image: TileImage? = null
    private var shader: BitmapShader? = null
    private val fill = Paint(Paint.ANTI_ALIAS_FLAG)
    private val img = Paint(Paint.ANTI_ALIAS_FLAG)
    private val border = Paint(Paint.ANTI_ALIAS_FLAG)
    private val scrim = Paint(Paint.ANTI_ALIAS_FLAG)
    private val text = Paint(Paint.ANTI_ALIAS_FLAG)
    private val rect = RectF()
    private val clip = Path()
    private var label: CharSequence = entry.label

    init {
        isFocusable = true
        isClickable = true
        isLongClickable = true
        defaultFocusHighlightEnabled = false
        setOnClickListener { onLaunch(entry) }
        setOnLongClickListener { onMove(entry); true }
    }

    override fun onSizeChanged(w: Int, h: Int, ow: Int, oh: Int) {
        if (w <= 0 || h <= 0) return
        text.typeface = Style.typeface(bold = true)
        text.textSize = Style.sp(13f)
        label = TextUtils.ellipsize(entry.label, android.text.TextPaint(text), w - Style.dp(16), TextUtils.TruncateAt.END)
        icons.load(entry, w, h) {
            image = it
            shader = null
            invalidate()
        }
    }

    override fun onFocusChanged(gainFocus: Boolean, direction: Int, previouslyFocusedRect: android.graphics.Rect?) {
        super.onFocusChanged(gainFocus, direction, previouslyFocusedRect)
        val ms = Style.animMs(120)
        val target = if (gainFocus) 1.05f else 1f
        if (ms > 0) {
            animate().scaleX(target).scaleY(target).translationZ(if (gainFocus) Style.dp(8) else 0f).setDuration(ms).start()
        } else {
            scaleX = target; scaleY = target; translationZ = if (gainFocus) Style.dp(8) else 0f
        }
        invalidate()
    }

    override fun onDraw(c: Canvas) {
        val r = Style.dp(Cfg.i("radius").toFloat())
        val bw = Style.dp(Cfg.i("border.width").toFloat())
        rect.set(bw / 2, bw / 2, width - bw / 2, height - bw / 2)
        fill.color = Cfg.color("color.panel")
        c.drawRoundRect(rect, r, r, fill)

        image?.let { im ->
            val s = shader ?: BitmapShader(im.bitmap, Shader.TileMode.CLAMP, Shader.TileMode.CLAMP).also { shader = it }
            img.shader = s
            c.drawRoundRect(rect, r, r, img)
        }

        if (Cfg.b("tile.labels")) {
            val bandH = Style.dp(30)
            val top = rect.bottom - bandH
            scrim.shader = LinearGradient(0f, top, 0f, rect.bottom, 0, Style.withAlpha(Cfg.color("color.bg"), 0.92f), Shader.TileMode.CLAMP)
            clip.reset()
            clip.addRoundRect(rect, r, r, Path.Direction.CW)
            c.save()
            c.clipPath(clip)
            c.drawRect(rect.left, top, rect.right, rect.bottom, scrim)
            c.restore()
            text.color = Cfg.color("color.text")
            c.drawText(label, 0, label.length, Style.dp(10), rect.bottom - Style.dp(9), text)
        }

        if (isFocused) {
            val animated = Cfg.b("border.anim") && Cfg.s("border.style") == "gradient" && Style.animMs(1) > 0
            Style.activeBorder(c, rect, r, bw, border, if (animated) Style.spin() else 0f)
            if (animated) postInvalidateOnAnimation()
        } else {
            border.style = Paint.Style.STROKE
            border.strokeWidth = Style.dp(1.5f)
            border.color = Cfg.color("color.border")
            c.drawRoundRect(rect, r, r, border)
        }
    }
}

/**
 * Los mosaicos en filas y columnas. Con `layout=master` el primero ocupa 2×2, como la ventana principal de un tiling.
 */
class Grid(ctx: Context) : ViewGroup(ctx) {
    private val cells = ArrayList<IntArray>()

    init {
        clipChildren = false
        clipToPadding = false
        val p = Style.dp(10).toInt()
        setPadding(p, p, p, p + Style.dp(24).toInt())
    }

    override fun onMeasure(widthSpec: Int, heightSpec: Int) {
        val w = MeasureSpec.getSize(widthSpec)
        val cols = Cfg.i("columns").coerceAtLeast(1)
        val gap = Style.dp(Cfg.i("gap")).toInt()
        val cellW = (w - paddingLeft - paddingRight - gap * (cols - 1)) / cols
        val cellH = (cellW * Cfg.i("tile.ratio") / 100f).toInt()
        val master = Cfg.s("layout") == "master" && childCount > 1 && cols >= 3

        cells.clear()
        val taken = ArrayList<BooleanArray>()
        fun row(r: Int): BooleanArray {
            while (taken.size <= r) taken.add(BooleanArray(cols))
            return taken[r]
        }
        var bottom = 0
        for (i in 0 until childCount) {
            val span = if (master && i == 0) 2 else 1
            var r = 0
            var col = 0
            search@ while (true) {
                for (c in 0..cols - span) {
                    if ((0 until span).all { dr -> (0 until span).all { dc -> !row(r + dr)[c + dc] } }) {
                        col = c
                        break@search
                    }
                }
                r++
            }
            for (dr in 0 until span) for (dc in 0 until span) row(r + dr)[col + dc] = true
            val left = paddingLeft + col * (cellW + gap)
            val top = paddingTop + r * (cellH + gap)
            val right = left + span * cellW + (span - 1) * gap
            val bot = top + span * cellH + (span - 1) * gap
            cells.add(intArrayOf(left, top, right, bot))
            getChildAt(i).measure(MeasureSpec.makeMeasureSpec(right - left, MeasureSpec.EXACTLY), MeasureSpec.makeMeasureSpec(bot - top, MeasureSpec.EXACTLY))
            bottom = maxOf(bottom, bot)
        }
        setMeasuredDimension(w, bottom + paddingBottom)
    }

    override fun onLayout(changed: Boolean, l: Int, t: Int, r: Int, b: Int) {
        for (i in 0 until childCount) {
            val c = cells.getOrNull(i) ?: continue
            getChildAt(i).layout(c[0], c[1], c[2], c[3])
        }
    }
}
