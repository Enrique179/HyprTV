"""Rehace los sprites del Slayer a partir de las 6 imágenes originales (JPG sobre fondo negro).

    python scripts/sprites.py <carpeta con las 6 imágenes>

Necesita `pip install pillow numpy`. Escribe en core/assets:
  - sprites_png/<pieza>.png   RGBA sin fondo, recortado, con los colores fijados a la paleta (resolución completa)
  - slayer.bin                las 8 piezas como índices de paleta con RLE: lo que lee core/src/render_slayer.rs

Por qué así:
  - El contorno exterior del dibujo también es negro y se funde con el fondo, así que no vale inundar el fondo desde el borde (se
    comería el contorno). La silueta se saca dilatando la figura R px (lo que miden las líneas) y rellenando los huecos.
  - Los colores se fijan a los 6 del dibujo: lo demás son transiciones del JPEG. Luego un filtro de moda 3x3 quita píxeles sueltos.
  - Los brazos se cortan por el codo (las filas de CORTES). Los pivotes y esos cortes también están en render_slayer.rs (`BRAZOS`):
    si se cambian aquí, hay que cambiarlos allí.
"""
import struct
import sys
from pathlib import Path

import numpy as np
from PIL import Image

# Nombre de cada archivo original (las fotos del móvil) -> pieza.
ORIGINALES = {
    "1001891074": "bota_der", "1001891076": "bota_izq", "1001891078": "brazo_izq",
    "1001891080": "brazo_der", "1001891082": "casco", "1001891084": "torso",
}
# Paleta fija (negro del contorno, verde, gris, visor, rojo, naranja). El orden lo asume render_slayer.rs (VISOR = 3, ROJO = 4, VERDE = 1).
PALETA = np.array([[0x03, 0x06, 0x02], [0x23, 0x6F, 0x09], [0x4F, 0x4F, 0x4F], [0x73, 0x6E, 0x6B], [0xEA, 0x08, 0x09], [0xF8, 0x57, 0x08]], np.int32)
TRANSPARENTE = 255
R = 4            # grosor de las líneas negras del dibujo, en píxeles
UMBRAL = 45      # por debajo de este brillo máximo, un píxel es fondo
# Orden de las piezas en slayer.bin, con las filas que se toman de cada imagen (None = entera). Solapan unos píxeles en el codo.
PIEZAS = [
    ("bota_izq", "bota_izq", None), ("bota_der", "bota_der", None),
    ("brazo_izq_alto", "brazo_izq", (0, 176)), ("brazo_izq_ante", "brazo_izq", (164, None)),
    ("brazo_der_alto", "brazo_der", (0, 276)), ("brazo_der_ante", "brazo_der", (264, None)),
    ("casco", "casco", None), ("torso", "torso", None),
]


def dilata(m, r):
    """Dilatación por disco (sin scipy)."""
    out = m.copy()
    h, w = m.shape
    for dy in range(-r, r + 1):
        for dx in range(-r, r + 1):
            if dx * dx + dy * dy > r * r:
                continue
            sh = np.zeros_like(m)
            sh[max(0, dy):min(h, h + dy), max(0, dx):min(w, w + dx)] = m[max(0, -dy):min(h, h - dy), max(0, -dx):min(w, w - dx)]
            out |= sh
    return out


def rellena_huecos(m):
    """Lo que no se alcanza desde el borde es figura (inundación del fondo exterior en 4 direcciones)."""
    fuera = np.zeros_like(m)
    fuera[0, :], fuera[-1, :], fuera[:, 0], fuera[:, -1] = ~m[0, :], ~m[-1, :], ~m[:, 0], ~m[:, -1]
    libre = ~m
    while True:
        n = fuera.copy()
        n[1:, :] |= fuera[:-1, :]; n[:-1, :] |= fuera[1:, :]
        n[:, 1:] |= fuera[:, :-1]; n[:, :-1] |= fuera[:, 1:]
        n &= libre
        if (n == fuera).all():
            return ~fuera
        fuera = n


def moda3(idx, k):
    """Filtro de moda 3x3: quita píxeles sueltos de otro color que deja el JPEG en los bordes."""
    h, w = idx.shape
    pad = np.pad(idx, 1, mode="edge")
    votos = np.zeros((k, h, w), np.int16)
    for dy in range(3):
        for dx in range(3):
            v = pad[dy:dy + h, dx:dx + w]
            for c in range(k):
                votos[c] += (v == c)
    votos[idx, np.arange(h)[:, None], np.arange(w)[None, :]] += 1  # en un empate gana el valor propio
    return votos.argmax(axis=0).astype(np.uint8)


def procesa(ruta):
    """Devuelve (índices de paleta con 255 = transparente, RGBA) de una imagen original, recortada a la figura."""
    im = np.asarray(Image.open(ruta).convert("RGB")).astype(np.int32)
    sil = rellena_huecos(dilata(im.max(axis=2) > UMBRAL, R))
    ys, xs = np.where(sil)
    sl = (slice(ys.min(), ys.max() + 1), slice(xs.min(), xs.max() + 1))
    im, sil = im[sl], sil[sl]
    idx = moda3(((im[:, :, None, :] - PALETA[None, None]) ** 2).sum(3).argmin(2).astype(np.uint8), len(PALETA))
    rgba = np.zeros(im.shape[:2] + (4,), np.uint8)
    rgba[..., :3], rgba[..., 3] = PALETA[idx], np.where(sil, 255, 0)
    return np.where(sil, idx, TRANSPARENTE).astype(np.uint8), rgba


def rle(a):
    """Pares (racha, índice), racha ≤ 255, recorriendo la imagen por filas."""
    flat, out, i = a.reshape(-1), bytearray(), 0
    while i < len(flat):
        j = i
        while j < len(flat) and flat[j] == flat[i] and j - i < 255:
            j += 1
        out += bytes([j - i, int(flat[i])])
        i = j
    return bytes(out)


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    origen = Path(sys.argv[1])
    assets = Path(__file__).resolve().parent.parent / "core" / "assets"
    (assets / "sprites_png").mkdir(parents=True, exist_ok=True)
    indices = {}
    for archivo, pieza in ORIGINALES.items():
        ruta = next(iter(origen.glob(archivo + ".*")), None)
        if ruta is None:
            sys.exit(f"falta {archivo}.* en {origen}")
        indices[pieza], rgba = procesa(ruta)
        Image.fromarray(rgba, "RGBA").save(assets / "sprites_png" / f"{pieza}.png")
    out = bytearray(b"HSPR") + bytes([len(PALETA)])
    for c in PALETA:
        out += bytes(int(v) for v in c)
    out += bytes([len(PIEZAS)])
    for nombre, base, filas in PIEZAS:
        a = indices[base]
        if filas:
            a = a[filas[0]:filas[1]]
        datos = rle(a)
        out += struct.pack("<HHI", a.shape[1], a.shape[0], len(datos)) + datos
        print(f"{nombre:16s} {a.shape[1]}x{a.shape[0]}  {len(datos)} bytes")
    (assets / "slayer.bin").write_bytes(out)
    print(f"slayer.bin: {len(out)} bytes")


if __name__ == "__main__":
    main()
