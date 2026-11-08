package com.hyprtv

import android.content.res.Resources
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.LinearGradient
import android.graphics.Matrix
import android.graphics.Paint
import android.graphics.RectF
import android.graphics.Shader
import android.graphics.Typeface
import android.os.SystemClock

/** Cómo se ve todo: lo que `Cfg` dice, traducido a pinceles. Se comparte entre el launcher, los ajustes y el borde de foco. */
object Style {
    private val density get() = Resources.getSystem().displayMetrics.density

    fun dp(v: Float): Float = v * density
    fun dp(v: Int): Float = v * density
    fun sp(v: Float): Float = v * Resources.getSystem().displayMetrics.scaledDensity

    fun typeface(bold: Boolean = false): Typeface {
        val family = when (Cfg.s("font")) {
            "sans" -> Typeface.SANS_SERIF
            "serif" -> Typeface.SERIF
            else -> Typeface.MONOSPACE
        }
        return Typeface.create(family, if (bold) Typeface.BOLD else Typeface.NORMAL)
    }

    fun withAlpha(color: Int, alpha: Float): Int = Color.argb((Color.alpha(color) * alpha).toInt().coerceIn(0, 255), Color.red(color), Color.green(color), Color.blue(color))

    /** Duración de una animación según `anim.speed` (100 = normal, 0 = sin animaciones). */
    fun animMs(base: Long): Long {
        val speed = Cfg.i("anim.speed")
        return if (speed <= 0) 0 else base * 100 / speed
    }

    fun gradient(r: RectF, rotation: Float = 0f): LinearGradient {
        val g = LinearGradient(
            r.left, r.top, r.right, r.bottom,
            intArrayOf(Cfg.color("color.a1"), Cfg.color("color.a2"), Cfg.color("color.a3")), null,
            // MIRROR para que al girar el degradado no queden bordes planos.
            Shader.TileMode.MIRROR,
        )
        if (rotation != 0f) g.setLocalMatrix(Matrix().apply { setRotate(rotation, r.centerX(), r.centerY()) })
        return g
    }

    /** Vueltas del degradado animado: un giro completo cada ~6 s a velocidad normal. */
    fun spin(): Float {
        val speed = Cfg.i("anim.speed").coerceAtLeast(1)
        return (SystemClock.uptimeMillis() * speed / 100 % 6000) / 6000f * 360f
    }

    /**
     * El borde de la ventana con foco, como `col.active_border` de Hyprland: degradado, sólido o con resplandor.
     * `r` es el rectángulo del trazo (ya con el medio grosor de margen).
     */
    fun activeBorder(c: Canvas, r: RectF, radius: Float, width: Float, p: Paint, rotation: Float = 0f) {
        p.style = Paint.Style.STROKE
        val style = Cfg.s("border.style")
        if (style == "glow") {
            p.shader = null
            for (i in 3 downTo 1) {
                p.color = withAlpha(Cfg.color("color.a2"), 0.16f * (4 - i) / 3f)
                p.strokeWidth = width + dp(3) * i
                c.drawRoundRect(r, radius + dp(1) * i, radius + dp(1) * i, p)
            }
        }
        p.strokeWidth = width
        if (style == "solid") {
            p.shader = null
            p.color = Cfg.color("color.a2")
        } else {
            p.color = Color.WHITE
            p.shader = gradient(r, rotation)
        }
        c.drawRoundRect(r, radius, radius, p)
        p.shader = null
    }
}
