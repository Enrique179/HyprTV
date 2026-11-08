# HyprTV (Android TV)

Tema tipo Hyprland para una Smart TV Android (KTC 2K, Google TV, Android 14, Realtek rtd2841a, **32 bits armeabi-v7a, 1,5 GB de RAM**)
sin root: launcher propio, borde de foco global, mascota que comenta lo que hay en pantalla y atajos del mando para apagarlo todo.
**Comentarios, textos de la interfaz, mensajes de commit y respuestas, en español.**

## Qué hace

- **Launcher** (`HomeActivity`): "espacios" `1:apps 2:media 3:sistema` como píldoras de waybar, mosaicos con huecos y esquinas redondeadas, borde
  activo degradado, reloj. Pulsación larga en un mosaico = pasarlo al siguiente espacio. Teclas `1`–`6` y canal ±/página ± cambian de espacio.
- **Servicio de accesibilidad** (`ScreenService`): dibuja el borde de foco sobre cualquier app, la mascota con su bocadillo, el aviso OSD de los
  atajos, intercepta la tecla Home (esta TV abre siempre Google TV con ella aunque HyprTV sea el inicio) y escucha los atajos.
- **Mascota**: el chibi del reproductor de música (`../Reproductor de musica`, 4 modelos) portado a Rust, más el **Slayer** (5º modelo, el Doom Slayer dibujado por el usuario: sprites suyos —casco, peto, brazos con cuchilla y botas— animados por el esqueleto, con sus propias animaciones; propio de HyprTV y por defecto). Habla solo en cambios de contexto, con
  pausa mínima entre frases, se esconde mientras suena audio y se va andando a la otra esquina si el foco cae donde está. Sarcástica (≈270 frases).
- **Rutina de aburrimiento** (`pet.idle`, tras `pet.idle.minutes` sin tocar el mando): se estira, pasea por la parte baja de la pantalla, baila
  sin música (8 coreografías del reproductor), saluda, se sienta, y **señala lo que hay enfocado** con un comentario sobre su título. Dura 25–45 s,
  se repite cada 1,5–3 min hasta 5 veces y luego se rinde con una frase de «abandono». Cualquier tecla o cambio de foco la corta y vuelve a su esquina
  andando. No arranca con el audio sonando, con la pantalla apagada ni en las horas de silencio.
- **Borde de foco por app**: por defecto solo en las apps de `ring.allow` (Ajustes y Play Store). Las que pintan su propio resaltado (YouTube, Xuper…)
  quedaban con un segundo marco desajustado. Modos `ring.mode`: `lista` / `todas` (menos `ring.deny`) / `ninguna`; selector en los ajustes y atajo
  `combo.ringapp` para la app de delante.
- **Ajustes** (`SettingsActivity`, también en la lista de apps de cualquier launcher): todo con el mando; `config.ini` editable desde el PC.
- Sin permiso de Internet: lo que se lee de la pantalla nunca sale de la TV.

## Estructura

```
core/                 Núcleo Rust (cdylib "hyprcore", JNI a mano con la crate `jni`; dibujo con tiny-skia)
  src/lib.rs          API en Rust puro (la que se prueba) + funciones JNI `Java_com_hyprtv_Native_*`
  src/config.rs       config.ini: claves, validación, 7 presets de paleta
  src/pose.rs         Pose + gestos (port de Pose.kt / Gestures del reproductor)
  src/look.rs         Los 5 modelos (el Slayer: `HairStyle::Helmet` + `TopStyle::Armor`; sus colores: ver el comentario de cada campo de `SLAYER`)
  src/render_slayer.rs Dibujo del Slayer con **los sprites del usuario** (casco, peto, 2 brazos cortados por el codo, 2 botas; los muslos se pintan): cada pieza
                      gira sobre su articulación y comparte `Cv`, esqueleto y sombra con render.rs. `Eyes` manda sobre el visor (Closed apaga, Wide enciende,
                      Happy = hablando) con una capa de máscara; de espaldas se cambia el visor y la luz roja del peto por verde. Los sprites se reducen
                      (filtro de caja) una vez por tamaño de búfer y se cachean los dos últimos. Las medidas (pivotes, cortes) están como constantes arriba.
  assets/slayer.bin   Las 8 piezas como índices de paleta (6 colores) con RLE, 44 KB; el núcleo no lleva decodificador PNG. Se genera con un script de Python
                      a partir de las 6 imágenes originales con `scripts/sprites.py` (el fondo negro se quita dilatando la figura ~4 px, que es lo que mide su
                      contorno, y rellenando huecos: así no se pierde el contorno negro exterior). Los pivotes y los cortes de los brazos viven en las
                      constantes de `render_slayer.rs` **y** en ese script: si se cambian en uno, cambiarlos en el otro. `assets/sprites_png`: las piezas en
                      RGBA a resolución completa y sin fondo; `assets/vista_previa_slayer.png`: la hoja de poses a 224 px (la del README).
  backup/             Solo en el disco del autor (no está en el repositorio): `render_slayer_vectorial.rs.txt`, el Slayer anterior todo con trazos.
                      Volver a él: copiarlo sobre `render_slayer.rs`, quitar `assets/slayer.bin` y devolver `look` a la firma de `figure`.
  src/slayer_moves.rs  Gestos y 8 bailes del Slayer (cabezazos, puñetazos, motosierra, recarga, músculos, patada, doble salto, dash). `pose::Moveset` agrupa gestos+bailes y `Brain::set_moveset` elige el de cada modelo
  src/render.rs       Dibujo del chibi (port de PetRenderer.kt) a un búfer RGBA premultiplicado
  src/brain.rs        Máquina de estados + rutina de aburrimiento (posición x, paseo, baile, señalar): anima solo cuando pasa algo y
                      al dormirse deja de pedir fotogramas (0 % de CPU). `pet_tick` devuelve retardo | evento<<16 | DIRTY
  src/moves.rs        Los 8 bailes del reproductor (`Moves.kt`), en pulsos de 0,5 s
  src/jokes.rs        Reglas de palabras clave → frases; cuándo hablar (pausa, horas de silencio, apps silenciadas); `Trigger::Activity`
                      con las frases de cada actividad («act:dance», «act:point»…)
  src/slayer.rs       Voz del Slayer (~190 frases borde con el usuario («recluta») y con el lore de Doom 2016/Eternal aplicado a la tele): solo habla con `pet.look=slayer`; `Voice` en jokes.rs
  src/keys.rs         Secuencias de teclas (atajos)
  examples/preview.rs Hoja con los 4 modelos en varias posturas, para compararlos a ojo
app/src/main/java/com/hyprtv/
  Native.kt           Puente JNI (los tiempos son SystemClock.uptimeMillis())
  Cfg.kt              Copia en memoria de la configuración (el archivo lo valida el núcleo)
  HyprApp.kt          Application + protección contra bucles de cierres + `DefaultHome` (abrir Google TV)
  Home.kt             Launcher: Background, Pill, Tile, Grid
  Apps.kt             Lista de apps, espacios por defecto, caché de iconos (LRU de 40, un hilo)
  SettingsActivity.kt Panel de ajustes y "aprender atajo"
  Overlays.kt         Ventanas TYPE_ACCESSIBILITY_OVERLAY: PetOverlay, RingOverlay, Osd
  ScreenService.kt    Eventos de ventana/foco, jokes, atajos, tecla Home
scripts/              build-rust.ps1, test-rust.ps1, deploy.ps1, sprites.py (rehace los sprites del Slayer; necesita `pip install pillow numpy`)
restaurar_tv.ps1      Devuelve la TV a Google TV y apaga el servicio (`-Desinstalar` quita también la app)
README.md             Presentación pública del proyecto
```

## Compilar, probar e instalar (Windows, PowerShell)

Las salidas de compilación van **fuera de OneDrive**: `%LOCALAPPDATA%\hyprtv-target` (Rust), `hyprtv-jni` (.so) y `hyprtv-build` (Gradle).

| Qué | Comando |
|---|---|
| Tests + clippy del núcleo (en el PC) | `.\scripts\test-rust.ps1` (filtro: `.\scripts\test-rust.ps1 jokes`) |
| Compilar el núcleo para armv7 + arm64 | `.\scripts\build-rust.ps1` (obligatorio tras tocar `core/`) |
| Compilar el APK e instalarlo en la TV | `.\scripts\deploy.ps1` (`-SinRust` si no tocó `core/`) |
| Hoja de poses | `cargo run -p hyprcore --example preview -- salida.png [lado]` (con `CARGO_TARGET_DIR` fuera de OneDrive; en la TV la mascota mide ~224 px) |

Para Gradle: `JAVA_HOME="C:\Program Files\Android\Android Studio\jbr"`. Versiones: AGP 9.4.1, Gradle 9.8.2, compileSdk 37, targetSdk 34, minSdk 26.
APK de ~1,3 MB, sin dependencias (nada de AndroidX ni Compose a propósito).

## En la TV (ADB inalámbrico)

- `adb` está en `%LOCALAPPDATA%\Android\Sdk\platform-tools` (no está en el PATH).
- Activar el servicio: `settings put secure enabled_accessibility_services com.hyprtv/com.hyprtv.ScreenService` y `settings put secure accessibility_enabled 1`.
- Inicio: `cmd package set-home-activity com.hyprtv/.HomeActivity` (queda como rol HOME). **No basta**: esta TV abre Google TV con la tecla Home,
  así que el servicio la intercepta (solo con el tema activado). Sin el servicio, Home abre Google TV y nada se rompe.
- Subir configuración: `adb push config.ini /sdcard/Android/data/com.hyprtv/files/config.ini` y reiniciar la app (se importa al arrancar si es más nuevo).
  **Cuidado: pisa los ajustes que el usuario haya cambiado con el mando.** Para pruebas, leer antes `adb shell cat` el archivo y volver a dejarlo como estaba.
- Volver a como estaba: `restaurar_tv.ps1` en la raíz del proyecto (`-Desinstalar` quita también la app).
- `adb shell input keyevent` **no pasa por el filtro de teclas de accesibilidad** y `sendevent` está denegado para el usuario shell:
  los atajos y la tecla Home solo se pueden probar con el mando real (mirar `logcat -s HyprTV`).
- El mando tiene Menú, Info, rojo/verde/amarillo, Silencio, etc. (`IR_KTCIR`).

## Decisiones y detalles que no se ven en el código

- **0 % de CPU en reposo**: la mascota anima a ~12 fps solo tras un evento; al dormirse `pet_tick` devuelve 0 y Kotlin no vuelve a programar nada.
  El servicio va por eventos (cambio de ventana, foco); no hay sondeos (el audio se vigila con `AudioPlaybackCallback`).
- **Eventos de ventana**: nuestras ventanas superpuestas también emiten `TYPE_WINDOW_STATE_CHANGED` desde `com.hyprtv` (hay que ignorarlos o se
  borra el borde nada más dibujarlo), y volver a una app ya abierta no emite ninguno: por eso se escucha también `TYPE_WINDOWS_CHANGED` y se
  confirma la app con `rootInActiveWindow` antes de hablar.
- **Borde de foco**: una ventana pequeña que sigue al nodo enfocado (no una capa a pantalla completa, que sobre vídeo costaría en una Mali-G31).
  Se ignoran nodos invisibles, minúsculos o casi a pantalla completa, y las pantallas propias (ya tienen su borde).
- **Mascota y bocadillo**: dos ventanas. La de la mascota es solo del tamaño del dibujo y se desplaza en x (`lp.x`, gravedad BOTTOM|START) siguiendo
  `Native.petX()`; el bocadillo es otra ventana que se quita y se vuelve a poner en cada frase, centrada sobre la cabeza. Una sola ventana
  «ajustada al contenido» con el bocadillo dentro salía recortada.
- **Rutina**: la decide Rust (`Brain::begin_routine`, `next_activity`); Kotlin solo la arranca (`ScreenService.maybeStartRoutine`) y traduce los
  eventos a frases. `onInput()` (teclas, foco, ventana) la interrumpe. La mascota no cambia de modo al hablar si está en rutina.
- **Atajos**: solo se consume la última tecla de la secuencia; las anteriores llegan a la app. Valores por defecto: Menú×3 (mascota) e Info×3 (tema);
  se reasignan en los ajustes ("aprender atajo": secuencia de 2 a 6 teclas que no sean flechas, OK, Atrás ni Home).
- **Tema apagado**: `HomeActivity` reenvía a Google TV, ring y mascota se ocultan y la tecla Home deja de interceptarse. Si el proceso se cierra
  dos veces en menos de un minuto, `Crashes.guard` apaga el tema solo.
- **Rust**: `panic = "unwind"` y cada función JNI captura pánicos (macro `guarded!`). El estado global es un `Mutex` único.

## Convenciones

- Escribir el código como el de alrededor: misma densidad de comentarios, nombres y estilo; comentarios en español que explican el *porqué*.
- Funciones puras en Rust y con pruebas en el PC (config, chistes, atajos, cerebro, dibujo). Clippy en 0 avisos.
- Los modelos y la paleta vienen del reproductor: si cambian allí, portar aquí a mano (`look.rs`, `render.rs`, `config.rs`).

## Qué se ha hecho (historial)

1. **2026-10-23 · Fases 1-2, limpiar la TV.** Se quitó telemetría y bloatware con `pm uninstall -k --user 0` (sin root, reversible: `adb shell cmd package
   install-existing <paquete>`). La lista de lo quitado está en `tv_paquetes_eliminados.txt`, **solo en el disco del autor** (es de su modelo de TV; no se publica).
2. **Fase 3 · HyprTV.** Launcher, borde de foco global, mascota con bocadillo, rutina de aburrimiento, atajos del mando y ajustes. Rust (núcleo) + Kotlin sin AndroidX.
3. **2026-10-25 · el Slayer vectorial.** 5º modelo de mascota, con voz propia (`slayer.rs`) y animaciones propias (`slayer_moves.rs`). Era el modelo por defecto.
4. **2026-11-08 · el Slayer con los sprites del usuario.** Se quitó el fondo negro de sus 6 dibujos (casco, peto, 2 brazos, 2 botas), se empaquetaron en
   `assets/slayer.bin` (44 KB) y `render_slayer.rs` los anima con el mismo esqueleto: brazos cortados por el codo, muslos pintados porque no hay dibujo,
   visor con capa de brillo, vista de espaldas cambiando el visor por verde. 49 pruebas OK y clippy en 0 avisos; el núcleo compila para armv7 y arm64 y el APK
   se generó (1,6 MB). **Se comprobó a ojo con `examples/preview` a 224 px, no en la TV.**
5. **Publicación.** Repositorio público limpiado de datos personales: fuera `local.properties` (ruta de Windows), `config_tv_antes.ini` (ajustes del usuario),
   `tv_paquetes_eliminados.txt`, `core/backup/`, compilaciones y APK (ver `.gitignore`). No hay claves, tokens, IP ni correos en el código; el APK de release se
   firma con la clave de depuración local, que no está en el proyecto.

## Qué queda por hacer (para retomar sin confusiones)

1. **Instalar en la TV el APK con los sprites. NO está hecho.** Al terminar la sesión `adb devices` salía vacío. Pasos: `adb mdns services` para ver el puerto actual
   (cambia), `adb connect <ip>:<puerto>` (si hace falta, `adb pair`), **leer antes `config.ini` de la TV** (`adb shell cat /sdcard/Android/data/com.hyprtv/files/config.ini`) y no
   subir ninguno de prueba, `.\scripts\deploy.ps1 -SinRust` (el núcleo ya está compilado; sin `-SinRust` lo recompila) y **reactivar el servicio de accesibilidad**
   (`adb install -r` lo deja apagado): los dos `settings put secure` de arriba. Comprobar con `adb shell settings get secure enabled_accessibility_services`.
2. **Verlo en la TV real.** Pendiente de mirar: si el Slayer se lee bien al tamaño por defecto (112 dp; el detalle de los sprites es denso, quizá `pet.size` 130-150),
   si anima con fluidez en el armv7 y cuánto tarda el primer fotograma tras arrancar o cambiar el tamaño (reduce 12 imágenes una vez por tamaño; no medido en la TV).
3. **Mejoras posibles, si el usuario las quiere:** dibujar los muslos y una vista de espaldas propia (ahora el visor se cambia por verde); pulir la costura del codo
   al doblarlo mucho (hoy se tapa con un disco gris); un sprite de ojos/visor encendido distinto en vez de la capa de brillo; cargar los demás modelos con sprites.
4. **Decisiones del usuario aún abiertas:** licencia del repositorio (ahora no tiene ninguna: por defecto todos los derechos reservados) y qué hacer con el arte, que se
   inspira en Doom (id Software/Bethesda).
5. **Si cambian los sprites originales:** `python scripts/sprites.py <carpeta con las 6 imágenes>`, pasar `.\scripts\test-rust.ps1`, `.\scripts\build-rust.ps1` y desplegar.

## Cómo retomar en una sesión nueva

Leer este archivo entero, luego `core/src/render_slayer.rs` (arriba del todo están todas las medidas). Antes de tocar la TV, mirar `adb devices` y la regla de no pisar `config.ini`.
