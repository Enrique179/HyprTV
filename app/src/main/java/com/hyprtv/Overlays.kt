package com.hyprtv

import android.content.Context
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.PixelFormat
import android.graphics.Rect
import android.graphics.RectF
import android.graphics.drawable.GradientDrawable
import android.os.Handler
import android.text.TextUtils
import android.view.Gravity
import android.view.View
import android.view.WindowManager
import android.widget.TextView
import java.nio.ByteBuffer

/**
 * Ventanas superpuestas del servicio de accesibilidad. Todas son `TYPE_ACCESSIBILITY_OVERLAY` sin foco ni toques: nunca
 * reciben el D-pad ni se interponen en lo que hay debajo.
 */
private fun overlayParams(w: Int, h: Int, noLimits: Boolean = false): WindowManager.LayoutParams {
    var flags = WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE or WindowManager.LayoutParams.FLAG_NOT_TOUCHABLE or
        WindowManager.LayoutParams.FLAG_LAYOUT_IN_SCREEN
    if (noLimits) flags = flags or WindowManager.LayoutParams.FLAG_LAYOUT_NO_LIMITS
    return WindowManager.LayoutParams(w, h, WindowManager.LayoutParams.TYPE_ACCESSIBILITY_OVERLAY, flags, PixelFormat.TRANSLUCENT)
}

/** El fotograma de la mascota, que dibuja el núcleo Rust en un búfer. */
private class PetView(ctx: Context, val bitmap: Bitmap) : View(ctx) {
    private val paint = Paint(Paint.FILTER_BITMAP_FLAG)

    override fun onMeasure(w: Int, h: Int) = setMeasuredDimension(bitmap.width, bitmap.height)

    override fun onDraw(c: Canvas) {
        c.drawBitmap(bitmap, 0f, 0f, paint)
    }
}

/**
 * La mascota. Su ventana es solo del tamaño del dibujo y se desplaza por la parte baja de la pantalla (para pasear en la
 * rutina de aburrimiento); el bocadillo es otra ventana que se coloca sobre su cabeza cada vez que habla. Así ninguna
 * ventana cambia de tamaño y no hay recortes.
 */
class PetOverlay(private val ctx: Context, private val wm: WindowManager, private val handler: Handler) {
    private val screenW = ctx.resources.displayMetrics.widthPixels
    private var sizePx = 0
    private var builtFor = ""
    private var bitmap: Bitmap? = null
    private var buf: ByteBuffer? = null
    private var petView: PetView? = null
    private var lp: WindowManager.LayoutParams? = null
    private var bubble: TextView? = null
    private var bubbleLp: WindowManager.LayoutParams? = null
    private var corner = "br"
    private var ticking = false
    private var bubbleAt = 0L

    /** Lo que cuenta la mascota cuando empieza algo de su rutina (ver `Native.EVENT_NAMES`). */
    var onEvent: ((Int) -> Unit)? = null

    /** Se llama cuando se queda dormida y ya no pide más fotogramas. */
    var onSleep: (() -> Unit)? = null

    var shown = false
        private set

    /** ¿Está en medio de su rutina de aburrimiento? */
    var routine = false
        private set

    private val tick = object : Runnable {
        override fun run() {
            val b = buf ?: return
            val r = Native.petTick(nowMs(), b)
            if (r and Native.DIRTY != 0) {
                b.rewind()
                bitmap?.copyPixelsFromBuffer(b)
                petView?.invalidate()
                follow()
            }
            val event = (r shr Native.EVENT_SHIFT) and Native.EVENT_MASK
            if (event != 0) {
                if (event == Native.EV_TIRED || event == Native.EV_BACK) routine = false
                onEvent?.invoke(event)
            }
            val delay = r and Native.DELAY_MASK
            if (delay > 0 && shown) {
                handler.postDelayed(this, delay.toLong())
            } else {
                // Dormida: no se pide ningún fotograma más hasta que algo la despierte (0 % de CPU).
                ticking = false
                routine = false
                onSleep?.invoke()
            }
        }
    }

    private val hideBubble = Runnable { fadeBubble() }

    private fun key() = "${Cfg.i("pet.size")}"
    private fun margin() = Style.dp(40).toInt()
    private fun bottomMargin() = Style.dp(24).toInt()

    /** Centro de la mascota, como fracción del ancho de la pantalla, cuando está en su esquina `c`. */
    private fun homeOf(c: String): Float {
        val half = sizePx / 2f + margin()
        return (if (c == "bl") half else screenW - half) / screenW
    }

    private fun sendWorld() {
        Native.petWorld(homeOf(corner), homeOf("bl"), homeOf("br"), nowMs())
    }

    /** Si cambió el tamaño se reconstruye; el modelo y la esquina se leen en caliente. */
    fun syncConfig() {
        if (shown && key() != builtFor) {
            hide()
            show()
        } else if (shown) {
            setCorner(Cfg.s("pet.corner"))
            kick()
        }
    }

    fun show() {
        if (shown) return
        sizePx = (Style.dp(112) * Cfg.i("pet.size") / 100f).toInt().coerceAtLeast(48)
        builtFor = key()
        val bmp = Bitmap.createBitmap(sizePx, sizePx, Bitmap.Config.ARGB_8888)
        bitmap = bmp
        buf = ByteBuffer.allocateDirect(sizePx * sizePx * 4)
        corner = Cfg.s("pet.corner")
        Native.petCreate(sizePx, sizePx, nowMs())
        sendWorld()

        val pet = PetView(ctx, bmp)
        petView = pet
        val params = overlayParams(sizePx, sizePx)
        params.gravity = Gravity.BOTTOM or Gravity.START
        params.x = (Native.petX() * screenW - sizePx / 2f).toInt()
        params.y = bottomMargin()
        params.title = "HyprTV pet"
        lp = params
        if (runCatching { wm.addView(pet, params) }.isSuccess) shown = true else return
        wake()
    }

    fun hide() {
        handler.removeCallbacks(tick)
        handler.removeCallbacks(hideBubble)
        ticking = false
        routine = false
        removeBubble()
        petView?.let { runCatching { wm.removeView(it) } }
        petView = null
        bitmap = null
        buf = null
        shown = false
    }

    /** Pone la ventana donde dice el núcleo (y el bocadillo con ella, si está a la vista). */
    private fun follow() {
        val p = lp ?: return
        val view = petView ?: return
        val x = (Native.petX() * screenW - sizePx / 2f).toInt()
        if (x != p.x) {
            p.x = x
            runCatching { wm.updateViewLayout(view, p) }
            if (bubble != null) placeBubble()
        }
    }

    private fun setCorner(c: String) {
        if (c == corner) return
        corner = c
        // El núcleo la hace caminar hasta la otra esquina; hay que seguir pidiendo fotogramas mientras tanto.
        sendWorld()
        kick()
    }

    fun kick() {
        if (!shown || ticking) return
        ticking = true
        handler.post(tick)
    }

    fun wake() {
        if (!shown) return
        Native.petWake(nowMs())
        routine = false
        kick()
    }

    /** Arranca la rutina de aburrimiento durante `ms` milisegundos. */
    fun startRoutine(ms: Long) {
        if (!shown || routine) return
        routine = true
        Native.petRoutine(nowMs(), ms)
        kick()
    }

    /** Alguien tocó el mando: la rutina se corta y vuelve a su esquina. */
    fun interrupt() {
        if (!shown || !routine) return
        routine = false
        Native.petInterrupt(nowMs())
        kick()
    }

    /** Hacia dónde está lo enfocado (fracción del ancho), para ir a señalarlo. */
    fun setFocusX(x: Float) {
        if (shown) Native.petFocus(x)
    }

    // ---- bocadillo ---------------------------------------------------------------------------

    fun say(text: String, ms: Long) {
        if (!shown) return
        removeBubble()
        val tv = TextView(ctx).apply {
            this.text = text
            typeface = Style.typeface()
            textSize = 16f
            setTextColor(Cfg.color("color.text"))
            maxWidth = Style.dp(300).toInt()
            maxLines = 4
            ellipsize = TextUtils.TruncateAt.END
            setPadding(Style.dp(12).toInt(), Style.dp(8).toInt(), Style.dp(12).toInt(), Style.dp(8).toInt())
            background = GradientDrawable().apply {
                setColor(Style.withAlpha(Cfg.color("color.panel"), 0.95f))
                cornerRadius = Style.dp(Cfg.i("radius"))
                setStroke(Style.dp(2).toInt(), Cfg.color("color.a2"))
            }
            alpha = 0f
        }
        val params = overlayParams(-2, -2)
        params.gravity = Gravity.BOTTOM or Gravity.START
        params.title = "HyprTV bubble"
        bubble = tv
        bubbleLp = params
        placeBubble(add = true)
        tv.animate().alpha(1f).setDuration(Style.animMs(180)).start()
        bubbleAt = nowMs()
        handler.removeCallbacks(hideBubble)
        handler.postDelayed(hideBubble, ms)
        Native.petTalk(nowMs(), ms.toInt())
        kick()
    }

    /** Centrado sobre la cabeza de la mascota y sin salirse de la pantalla. */
    private fun placeBubble(add: Boolean = false) {
        val tv = bubble ?: return
        val params = bubbleLp ?: return
        tv.measure(View.MeasureSpec.makeMeasureSpec(Style.dp(300).toInt(), View.MeasureSpec.AT_MOST), View.MeasureSpec.UNSPECIFIED)
        val w = tv.measuredWidth
        val edge = Style.dp(16).toInt()
        val centre = (Native.petX() * screenW).toInt()
        params.x = (centre - w / 2).coerceIn(edge, (screenW - w - edge).coerceAtLeast(edge))
        params.y = bottomMargin() + (sizePx * 0.80f).toInt()
        if (add) {
            if (runCatching { wm.addView(tv, params) }.isFailure) bubble = null
        } else {
            runCatching { wm.updateViewLayout(tv, params) }
        }
    }

    private fun removeBubble() {
        handler.removeCallbacks(hideBubble)
        bubble?.let {
            it.animate().cancel()
            runCatching { wm.removeView(it) }
        }
        bubble = null
        bubbleLp = null
    }

    private fun fadeBubble() {
        val tv = bubble ?: return
        tv.animate().cancel()
        tv.animate().alpha(0f).setDuration(Style.animMs(200)).withEndAction { if (bubble === tv) removeBubble() }.start()
    }

    /** Navegar rápido cancela el bocadillo: no hay que leer nada mientras uno se mueve por el menú. */
    fun dismissBubbleIfStale() {
        if (bubble != null && nowMs() - bubbleAt > 3000) fadeBubble()
    }

    // ---- no tapar lo enfocado ----------------------------------------------------------------

    private fun footprint(c: String, screenH: Int): Rect {
        val cx = homeOf(c) * screenW
        val top = screenH - bottomMargin() - sizePx
        val r = Rect((cx - sizePx / 2f).toInt(), top, (cx + sizePx / 2f).toInt(), screenH - bottomMargin())
        if (c == corner) bubbleLp?.let { b -> bubble?.let { tv -> r.union(Rect(b.x, screenH - b.y - tv.measuredHeight, b.x + tv.measuredWidth, screenH - b.y)) } }
        return r
    }

    /** Si lo enfocado cae justo donde está la mascota, se va andando a la otra esquina para no taparlo. */
    fun avoid(focus: Rect, screenH: Int) {
        if (!shown || routine) return
        val want = Cfg.s("pet.corner")
        val other = if (want == "br") "bl" else "br"
        val pick = when {
            !Rect.intersects(footprint(want, screenH), focus) -> want
            !Rect.intersects(footprint(other, screenH), focus) -> other
            else -> want
        }
        setCorner(pick)
    }
}

/** El borde de foco que sigue al elemento enfocado en cualquier app: un marco pequeño, no una capa a pantalla completa. */
class RingOverlay(private val ctx: Context, private val wm: WindowManager) {
    private val view = RingView(ctx)
    private var added = false
    private val lp = overlayParams(1, 1, noLimits = true).apply {
        gravity = Gravity.TOP or Gravity.START
        title = "HyprTV ring"
    }

    fun show(r: Rect) {
        val pad = Style.dp(Cfg.i("ring.padding")).toInt()
        val bw = Style.dp(Cfg.i("border.width")).toInt()
        val glow = if (Cfg.s("border.style") == "glow") Style.dp(10).toInt() else 0
        val m = bw + glow
        view.pad = pad
        view.margin = m
        lp.x = r.left - pad - m
        lp.y = r.top - pad - m
        lp.width = r.width() + 2 * (pad + m)
        lp.height = r.height() + 2 * (pad + m)
        if (!added) {
            runCatching { wm.addView(view, lp) }.onSuccess { added = true }
        } else {
            runCatching { wm.updateViewLayout(view, lp) }
        }
        view.invalidate()
    }

    fun hide() {
        if (added) {
            runCatching { wm.removeView(view) }
            added = false
        }
    }
}

private class RingView(ctx: Context) : View(ctx) {
    var pad = 0
    var margin = 0
    private val paint = Paint(Paint.ANTI_ALIAS_FLAG)
    private val rect = RectF()

    override fun onDraw(c: Canvas) {
        val bw = Style.dp(Cfg.i("border.width"))
        val inset = margin - bw / 2f
        rect.set(inset, inset, width - inset, height - inset)
        val radius = Style.dp(Cfg.i("radius")) + pad
        val animated = Cfg.b("border.anim") && Cfg.s("border.style") == "gradient" && Style.animMs(1) > 0
        Style.activeBorder(c, rect, radius, bw, paint, if (animated) Style.spin() else 0f)
        if (animated) postInvalidateOnAnimation()
    }
}

/** Aviso corto arriba en el centro ("mascota: apagada"). */
class Osd(private val ctx: Context, private val wm: WindowManager, private val handler: Handler) {
    private var view: TextView? = null
    private val hide = Runnable { remove() }

    fun show(text: String) {
        remove()
        val tv = TextView(ctx).apply {
            this.text = text
            typeface = Style.typeface(bold = true)
            textSize = 20f
            setTextColor(Cfg.color("color.text"))
            setPadding(Style.dp(20).toInt(), Style.dp(10).toInt(), Style.dp(20).toInt(), Style.dp(10).toInt())
            background = GradientDrawable().apply {
                setColor(Style.withAlpha(Cfg.color("color.panel"), 0.95f))
                cornerRadius = Style.dp(Cfg.i("radius").coerceAtLeast(10))
                setStroke(Style.dp(2).toInt(), Cfg.color("color.a2"))
            }
        }
        val p = overlayParams(-2, -2).apply {
            gravity = Gravity.TOP or Gravity.CENTER_HORIZONTAL
            y = Style.dp(32).toInt()
            title = "HyprTV osd"
        }
        if (runCatching { wm.addView(tv, p) }.isSuccess) {
            view = tv
            handler.postDelayed(hide, 1800)
        }
    }

    private fun remove() {
        handler.removeCallbacks(hide)
        view?.let { runCatching { wm.removeView(it) } }
        view = null
    }
}
