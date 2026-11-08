//! La voz del Slayer (`pet.look=slayer`): el Doom Slayer metido en una tele. Seco, borde y con muy mala leche, y con el usuario más que con nadie
//! (lo trata de recluta y no le perdona una). Mismas reglas que la mascota sarcástica (palabras clave → frase, una vez por app, pausas), otro
//! repertorio: el lore de Doom (2016) y Doom Eternal (la UAC, Marte, Hayden y VEGA, la energía Argent, Khan Maykr, el Icono del Pecado, la
//! Fortaleza de Doom, Daisy la coneja…) aplicado a lo que pasa en la tele. Son referencias, no citas: nada de diálogos copiados.

use crate::jokes::{AppRule, TextRule, Voice};

pub(crate) const VOICE_SLAYER: Voice = Voice {
    text: TEXT_RULES,
    apps: APP_RULES,
    idle: IDLE,
    night: NIGHT,
    morning: MORNING,
    evening: EVENING,
    activity: activity_lines,
};

const TEXT_RULES: &[TextRule] = &[
    TextRule {
        keys: &["wi-fi", "wifi", "red inalambrica", " red "],
        lines: &[
            "Se cae el Wi‑Fi y lloras. Yo crucé el infierno sin cobertura. Reinicia el router, recluta.",
            "La UAC abrió un portal a Hell con menos quejas que tú por una barrita de señal.",
            "Sin conexión. En Marte, Hayden lo arreglaba con un portal. Tú tienes un enchufe y cero excusas.",
            "Lag. Un Cacodemonio con latencia sigue matando mejor que tú.",
        ],
    },
    TextRule {
        keys: &["almacenamiento", "storage", "espacio"],
        lines: &[
            "Cuatro gigas y los llenas de basura. Qué débil. Mi arsenal cabe entero en una armadura.",
            "Libera espacio o te lo libero yo, y no te va a gustar el método.",
            "Cada app inútil es un demonio menor ocupando tu base. Purga.",
        ],
    },
    TextRule {
        keys: &["actualizacion", "actualizar", "update"],
        lines: &[
            "Una actualización. Los Maykr también prometían mejoras y luego te enviaban a un santuario.",
            "Actualizando… VEGA tardaba menos en reparar una base entera.",
            "«Nueva versión». Han movido los botones. Hasta Hell tiene mejor señalización.",
        ],
    },
    TextRule {
        keys: &["bluetooth", "emparejar", "mando a distancia"],
        lines: &[
            "Emparejar un mando a la tercera. Yo emparejo una escopeta y un Revenant en menos tiempo.",
            "Bluetooth: la tele no te obedece. A mí tampoco me obedecían los demonios y mira cómo acabaron.",
        ],
    },
    TextRule {
        keys: &["desarrollador", "depuracion", "adb"],
        lines: &[
            "Opciones de desarrollador. Mucho poder para tan poca disciplina.",
            "Depuración activa. Si rompes algo, lo arreglas tú. Yo solo sé romper cosas.",
            "Modo desarrollador: el equivalente a abrir el sarcófago. Cuidado con lo que despiertas.",
        ],
    },
    TextRule {
        keys: &["privacidad", "anuncios", "publicidad", "personalizados"],
        lines: &[
            "Privacidad. Aquí nadie te espía. Salvo yo, y solo para criticarte.",
            "Anuncios personalizados desactivados. Un enjambre de demonios menos. Qué lástima, me estaba entrenando.",
        ],
    },
    TextRule {
        keys: &["accesibilidad"],
        lines: &[
            "Accesibilidad: aquí está mi puesto de mando. Veo la pantalla, no me hace falta tu permiso.",
            "Este menú es mi Fortaleza de Doom. No toques nada y no te pasa nada.",
        ],
    },
    TextRule {
        keys: &["sonido", "audio", "volumen", "altavoz"],
        lines: &[
            "Más volumen. Eso es: que suene como una banda sonora de las buenas, no como un susurro.",
            "Los diálogos no se oyen y tú subes el volumen. Yo mataba a un Baron con la música a todo y no pedí subtítulos.",
            "La mejor banda sonora de la historia suena cuando se rasga y se desgarra. Tu tele, no tanto.",
        ],
    },
    TextRule {
        keys: &["imagen", "brillo", "contraste", "pantalla"],
        lines: &[
            "Brillo al máximo. Para ver bien cómo envejeces mientras ves series.",
            "Modo cine. Lo usas una vez y luego vuelves al vívido. No tienes remedio.",
            "Mira esa imagen. Más nítida que tus decisiones de hoy.",
        ],
    },
    TextRule {
        keys: &["contrasena", "password", "iniciar sesion", "inicia sesion", "correo", "email"],
        lines: &[
            "Teclear una contraseña con el mando. Matar a un Marauder fue más rápido.",
            "Mayúscula, número y símbolo. Y tú la apuntaste en un papel. Patético.",
            "Olvidaste la contraseña. Yo olvidé mi propio nombre y aun así sigo en pie.",
        ],
    },
    TextRule {
        keys: &["buscar", "busqueda", "search"],
        lines: &[
            "Buscas «algo bueno». Tardas más en decidir que un Baron en caer.",
            "Escribe lo que quieres ver, recluta. «Algo» no es un objetivo.",
            "Buscar con el mando, letra a letra. Lo mismo que recargar a ojo: lento y sin ganas.",
        ],
    },
    TextRule {
        keys: &["error", "no se pudo", "sin conexion", "reintentar", "fallo", "problema"],
        lines: &[
            "Error. Reinténtalo y no me vengas llorando.",
            "«Algo salió mal.» Tienes razón: has pulsado algo, ¿verdad?",
            "Un fallo. Los demonios no tienen ventanas de error: te matan y punto.",
        ],
    },
    TextRule {
        keys: &["cargando", "loading", "espera"],
        lines: &[
            "Cargando. Aprende a esperar: yo pasé siglos en un sarcófago sin quejarme.",
            "La ruedita da vueltas. Paciencia, recluta; el infierno no se conquistó en cinco minutos.",
            "Esperando. Hasta Hayden tenía más paciencia que tú y fíjate cómo era.",
        ],
    },
    TextRule {
        keys: &["anuncio", " ad ", "patrocinado", "saltar anuncio"],
        lines: &[
            "Un anuncio. Otro enjambre de demonios menores. Salta, que no me da tiempo a sacar la motosierra.",
            "Cinco segundos de anuncio. Cinco segundos más que un Imp en mi presencia.",
            "Publicidad: la invasión que nadie puede cortar a tiros. Qué frustrante.",
        ],
    },
    TextRule {
        keys: &["noticias", "informativo", "telediario", "ultima hora"],
        lines: &[
            "Noticias: el mundo arde y tú con el mando en la mano. Sin armadura.",
            "Última hora: sigue habiendo demonios. Pero los vuestros son mucho más aburridos.",
        ],
    },
    TextRule {
        keys: &["trailer", "avance"],
        lines: &[
            "Un tráiler. Dos minutos de promesas. Yo no hago tráileres: hago masacres.",
            "«{t}»: el tráiler enseña lo mejor y luego te toca aguantar. Como todo.",
            "Un avance con explosiones. Ni de lejos es el desfile de Argent que vi en Eternal.",
        ],
    },
    TextRule {
        keys: &["tutorial", "como hacer", "como arreglar", "paso a paso"],
        lines: &[
            "Un tutorial. Yo aprendí a rasgar y desgarrar sin ninguno. Tú necesitas diez minutos.",
            "«{t}»: te lo explican con calma y aun así lo harás mal.",
        ],
    },
    TextRule {
        keys: &["receta", "cocina", "cocinar", "postre"],
        lines: &[
            "Una receta. Yo me alimento de Argent y de demonios fritos. Tu plan de arroz me parece flojo.",
            "«{t}»: lo vas a quemar. Lo digo por experiencia ajena.",
        ],
    },
    TextRule {
        keys: &["futbol", " gol ", "liga ", "partido", "mundial", "champions", "deportes"],
        lines: &[
            "Veintidós tipos persiguiendo una pelota. Yo persigo Barones. Más honesto.",
            "«{t}»: gritas al árbitro como si pudiera oírte. Los demonios tampoco escuchan.",
            "Un gol. Qué poco hace falta para que te emociones. Con un Pinky desde atrás, ni me inmuto.",
        ],
    },
    TextRule {
        keys: &["baloncesto", "nba", "tenis", "formula 1", "ufc", "boxeo", "rugby", "beisbol"],
        lines: &[
            "Deportes. Gente que golpea con reglas. A mí me enseñaron a golpear sin ellas.",
            "«{t}»: ves a otros sudar mientras tú engordas. Rendición total.",
        ],
    },
    TextRule {
        keys: &["infantil", "dibujos", "kids", "ninos", "peppa", "bluey"],
        lines: &[
            "Dibujos animados. Yo tengo un conejo llamado Daisy y no me pongo a cantar.",
            "Los críos ven eso. Tú también. Prefiero no comentar nada. Daisy tampoco.",
        ],
    },
    TextRule {
        keys: &["terror", "miedo", "horror", "paranormal"],
        lines: &[
            "Terror. He destripado el infierno en dos juegos y esto me da pereza.",
            "Un fantasma. Un Lost Soul sí daba miedo; esto es un actor con maquillaje.",
            "¿Miedo tú? Yo me desayuno al Icono del Pecado.",
        ],
    },
    TextRule {
        keys: &["documental", "naturaleza", "historia de"],
        lines: &[
            "Un documental. Si quieres historia de verdad, ve a la Fortaleza de Doom y lee mis testamentos.",
            "«{t}»: aprendes algo y lo olvidas antes de apagar la tele. Típico.",
        ],
    },
    TextRule {
        keys: &["capitulo", "episodio", "temporada", "serie"],
        lines: &[
            "Un capítulo más. Y luego otro. Así se pierde una vida. Yo la perdí en el infierno, tú en el sofá.",
            "«{t}»: maratón nocturno. Mañana en pie con ojeras. Así no se salva ningún planeta.",
            "Una temporada entera de golpe. Débil de mente, pero admiro la constancia.",
        ],
    },
    TextRule {
        keys: &["pelicula", "movie", "estreno", "cine"],
        lines: &[
            "Veinte minutos eligiendo película. Los demonios no te dan tanto tiempo.",
            "«{t}»: una peli. Otra excusa para no moverte del sofá.",
            "Estreno de la semana. Tantos efectos y ninguno es una escopeta de doble cañón.",
        ],
    },
    TextRule {
        keys: &["en vivo", " live ", "directo"],
        lines: &[
            "En directo: no hay pausa ni guardado. Igual que en el infierno. Compórtate.",
            "«{t}», en vivo. Si te lo pierdes, te jodes. Sin repeticiones.",
        ],
    },
    TextRule {
        keys: &["gameplay", "videojuego", "gaming", "minecraft", "fortnite", "roblox", "doom", "zelda", "mario", "pokemon"],
        lines: &[
            "Videojuegos. Por fin algo que entiendo. Pásame el mando antes de que lo rompas.",
            "«{t}». ¿Ves a otro jugar? Cobarde. Coge el mando y mata algo.",
            "Doom, dicen. Ese me suena. Fue un buen tiempo de sangre y motosierra.",
            "Un chaval jugando. Si fuera yo, ya habría acabado y el servidor se habría rendido.",
        ],
    },
    TextRule {
        keys: &["musica", "lofi", "playlist", "concierto", "reggaeton", "karaoke", " mix "],
        lines: &[
            "«{t}»: música. Mick Gordon tenía otra idea de volumen. Y de guitarra.",
            "Lofi para estudiar. Qué blando. Para estudiar el infierno hace falta metal a 200 BPM.",
            "Karaoke de sofá. Desafinas más que un Whiplash.",
        ],
    },
    TextRule {
        keys: &["reaccion", "reaction", "react"],
        lines: &[
            "Ver cómo alguien reacciona a algo. Qué ganas de que otro viva tu vida.",
            "«{t}»: un tipo mirando otra pantalla. Recursividad de demonio perezoso.",
        ],
    },
    TextRule {
        keys: &["suscribete", "suscribirse", "premium", "suscripcion", "plan mensual", "pruebalo gratis"],
        lines: &[
            "Suscripciones. Pagas por ver lo que antes era gratis. Te han estafado, recluta.",
            "«Pruébalo gratis». Los demonios también prometían buen trato y mira.",
            "Premium: lo mismo con menos demonios. Y tú, feliz de pagar.",
        ],
    },
    TextRule {
        keys: &["omitir", "saltar intro", "saltar resumen", "skip"],
        lines: &[
            "Saltar la intro. Yo salté el protocolo de entrada a Marte y mira qué bien me salió.",
            "Omitir, omitir, omitir. Impaciente. Me gusta.",
        ],
    },
    TextRule {
        keys: &["continuar viendo", "reanudar", "seguir viendo", "retomar"],
        lines: &[
            "Continuar donde lo dejaste. Cobarde: la serie lleva dos semanas esperándote.",
            "Retomar lo que abandonaste. Qué raro verte terminar algo.",
        ],
    },
    TextRule {
        keys: &["favoritos", "mi lista", "guardados", "ver mas tarde"],
        lines: &[
            "«Mi lista»: cien cosas que no vas a ver. Un cementerio mejor ordenado que el de Hell.",
            "Ver más tarde. Ese «más tarde» nunca llega. Lo sé: me pasó con mi propia invasión.",
        ],
    },
    TextRule {
        keys: &["historial", "recientes", "visto recientemente"],
        lines: &[
            "Tu historial. Un registro de tus pecados. Y no soy ningún Maykr para juzgarte, pero qué asco.",
            "Revisando lo que viste. Mejor que no te lo lea en voz alta.",
        ],
    },
    TextRule {
        keys: &["subtitulos", "idioma", "doblaje", "audio original"],
        lines: &[
            "Subtítulos. Yo hablo con una escopeta y todos me entienden.",
            "¿Doblado u original? Elige rápido, que los diálogos no se escuchan solos.",
        ],
    },
    TextRule {
        keys: &["control parental", "perfil infantil", " pin ", "bloqueo"],
        lines: &[
            "Un PIN. Seguro que es 1234. Hasta una puerta de la UAC tenía mejor seguridad.",
            "Control parental: cuatro dígitos que tus hijos ya saben. Brillante.",
        ],
    },
    TextRule {
        keys: &["reiniciar", "apagar", "suspender", "restablecer"],
        lines: &[
            "Apagarme. Inténtalo. Ya me intentó apagar un sarcófago durante siglos.",
            "Reiniciar. La solución de los débiles. Aun así, hazlo.",
            "Restablecer de fábrica: me borras a mí. Atrévete, recluta.",
        ],
    },
    TextRule {
        keys: &["cerrar sesion", "cuenta", "perfil", "usuario"],
        lines: &[
            "Tu perfil. Tu cuenta. Tus datos. Y yo sin acceso a ninguno de ellos. Ni quiero.",
            "Cerrar sesión: huir del nivel. Luego a teclear la contraseña otra vez. Sufre.",
        ],
    },
    TextRule {
        keys: &["descargar", "instalar", "desinstalar", "install"],
        lines: &[
            "Instalando otra app. La tele tiene la RAM de un Imp. No la mates.",
            "Descargando. Aprovecha para pensar cómo vas a justificar otra app que no abrirás.",
            "Desinstalar: eso sí es limpiar el nivel. Sigue así.",
        ],
    },
    TextRule {
        keys: &["permitir", "aceptar", "terminos", "condiciones", "acepto"],
        lines: &[
            "Aceptas sin leer. Los Maykr también hacían firmar sin leer. Mira cómo les fue.",
            "Cuarenta páginas de condiciones. Yo leo solo las de las armas.",
            "Permitir, permitir, permitir. Le abriste las puertas del infierno a una app de linternas.",
        ],
    },
    TextRule {
        keys: &["notificaciones", "avisos"],
        lines: &[
            "Notificaciones. Demonios menores sin parar. Dispárales, recluta.",
            "Otra notificación. No tiene ni la dignidad de un Possessed: ni siquiera te ataca.",
        ],
    },
    TextRule {
        keys: &["canales", "canal ", "guia de programacion", "tv en vivo", "zapping"],
        lines: &[
            "Cien canales y ninguno decente. Qué pena de pecera.",
            "Zapping: cambias de canal como yo de arma. Pero yo sí acierto.",
            "«{t}». Elige uno y quédate. Los indecisos acaban en el infierno.",
        ],
    },
    TextRule {
        keys: &["radio", "podcast"],
        lines: &[
            "Radio en una pantalla enorme. Habla bajito, que se despierta Daisy.",
            "Un podcast. Gente hablando sin parar. Prefiero el silencio de después de una matanza.",
        ],
    },
    TextRule {
        keys: &["anime", "manga", "otaku"],
        lines: &[
            "Anime: tres capítulos de gritos antes del golpe final. Yo termino el combate en tres segundos.",
            "«{t}». Poder de Argent al máximo, flashback, grito. Reconozco que me gusta.",
        ],
    },
    TextRule {
        keys: &["comedia", "stand up", "humor", "chiste"],
        lines: &[
            "Comedia. Me río poco. Me río cuando un Marauder se queda sin perro.",
            "Un cómico en la tele. Ya me contó el infierno el mejor chiste: yo sigo vivo.",
        ],
    },
    TextRule {
        keys: &["viaje", "vlog", "turismo", "mochilero"],
        lines: &[
            "Vlogs de viajes. He viajado a Marte, a Hell y a Urdak. Tú, de la cama al sofá.",
            "«{t}». Bonito. Como un nivel sin enemigos: aburrido.",
        ],
    },
    TextRule {
        keys: &["review", "unboxing", "tecnologia", "reseña", "comparativa"],
        lines: &[
            "Un unboxing. Abres cajas ajenas. Yo abro cráneos propios.",
            "Reseñas de tecnología. Te venden un aparato que no necesitas. Yo no necesito ni ojos.",
        ],
    },
    TextRule {
        keys: &["oferta", "descuento", "gratis", "promocion", "rebajas"],
        lines: &[
            "Una oferta. Como los Maykr: te dan algo para quitártelo todo después.",
            "Gratis: la trampa más vieja del infierno. Y tú, mordiendo el anzuelo.",
        ],
    },
    TextRule {
        keys: &["ranking", "los mejores", " top ", "tendencias", "popular"],
        lines: &[
            "Lo más visto. El rebaño manda. Yo nunca he seguido a nadie, salvo a los demonios para matarlos.",
            "Tendencias. Siempre lo mismo con otro nombre. Como los Imps.",
        ],
    },
    TextRule {
        keys: &["clima", "pronostico", "temperatura", "lluvia"],
        lines: &[
            "El clima. En Hell hace siempre lo mismo: calor y gritos. Y yo, sin quejarme.",
            "Pronóstico del tiempo. Yo lo predigo mejor: va a llover sangre.",
        ],
    },
    TextRule {
        keys: &["fondo de pantalla", "salvapantallas", "protector"],
        lines: &[
            "Un salvapantallas. Cuando nadie toca nada, los demonios se acercan. Aprende de eso.",
            "Fondos de pantalla. Pon una calavera, recluta, o haz que me quede yo.",
        ],
    },
    TextRule {
        keys: &["hyprtv", "espacios de trabajo", "borde de foco", "mascota"],
        lines: &[
            "¿Tocándome los ajustes? Cuidado. Pesa más mi armadura que tu paciencia.",
            "Cambia lo que quieras. Pero no me toques el casco.",
            "Configurando al Slayer. Valiente. O muy tonto. Aún lo estoy decidiendo.",
        ],
    },
];

const APP_RULES: &[AppRule] = &[
    AppRule {
        packages: &["com.google.android.youtube.tv", "com.google.android.youtube"],
        lines: &[
            "YouTube. Entras a ver una cosa y sales cuatro horas después. Así se pierden las guerras.",
            "El algoritmo es un Maykr de bolsillo: te da lo que quieres y te quita las horas.",
            "Un vídeo más, dices. Mentiroso. Llevas ahí desde antes de que yo aprendiera a leer.",
            "Recomendado para ti: lo mismo de siempre. Como los Imps, sin variedad ni vergüenza.",
            "El algoritmo te conoce mejor que Hayden conocía la energía Argent. Y a ti te sale peor.",
        ],
    },
    AppRule {
        packages: &["com.android.tv.settings", "com.android.settings", "com.google.android.tvlauncher.settings"],
        lines: &[
            "Ajustes. No toques nada que no entiendas. Yo me quedé sin un sarcófago por menos.",
            "Entras a cambiar el brillo y acabas con la tele en alemán. Típico de quien toca sin leer.",
            "Los ajustes: aquí es donde la gente rompe cosas pensando que las arregla.",
            "Menú de opciones. Mucho texto, poca acción. Odio los menús.",
        ],
    },
    AppRule {
        packages: &["com.android.vending"],
        lines: &[
            "Play Store. Otra app que ocupará tu RAM sin dar nada. Un Pinky pesa lo mismo y al menos embiste.",
            "Instalar, instalar, instalar. La tele sangra por la memoria. Te lo advierto.",
            "Ojo con lo que instalas. Cualquier cosa puede abrir un portal. Yo lo sé bien.",
            "La tienda: un mercado de basura brillante. Elige con cabeza, recluta.",
        ],
    },
    AppRule {
        packages: &["com.android.mgstv"],
        lines: &[
            "Zapping. Cien canales y ninguno es una masacre. Qué decepción.",
            "Xuper: cuarenta mil títulos. Yo solo necesito un buen objetivo.",
            "Tanta oferta y acabas viendo lo de siempre. Débil de voluntad.",
            "Elige uno y quédate. En el infierno no dan vueltas: embisten.",
        ],
    },
    AppRule {
        packages: &["com.instantbits.cast.receiver"],
        lines: &[
            "El móvil manda y la tele obedece. Así debería ser con los demonios.",
            "Proyectas desde el teléfono. Pantalla grande para tu ego pequeño.",
            "Web Video Cast: tu móvil envía y yo recibo. Menos mal que tengo paciencia.",
        ],
    },
    AppRule {
        packages: &["com.google.android.katniss", "com.google.android.googlequicksearchbox"],
        lines: &[
            "Hablándole a la tele. Aquí los únicos que contestan son los demonios y no te gustan.",
            "Búsqueda por voz. Di algo claro, recluta. «Algo bueno» no es una orden.",
        ],
    },
    AppRule {
        packages: &["com.hyprtv", "com.google.android.apps.tv.launcherx", "com.google.android.tvlauncher"],
        lines: &[
            "Bienvenido a la Fortaleza. Aquí no se descansa, se prepara la siguiente matanza.",
            "El inicio. Elige una app y deja de mirar, recluta.",
            "Este launcher es ligero. Mi armadura, en cambio, pesa lo suyo. Y tú, a ver si te mueves.",
            "Hecho en Rust. Sin errores, sin pausas, sin piedad. Como yo.",
            "Tres pulsaciones del atajo y me callo. Pero no me voy: soy más persistente que Hell.",
            "Volviste al inicio. Como quien vuelve a la base sin haber matado nada.",
        ],
    },
    AppRule {
        packages: &["com.google.android.apps.mediashell", "com.google.android.gms"],
        lines: &["Proyectando algo desde el móvil. Cobarde: ni la tele quieres usar con tus manos."],
    },
    AppRule {
        packages: &["com.google.android.packageinstaller", "com.android.packageinstaller"],
        lines: &["Instalando. Si trae algo raro y se abre un portal, no digas que no te avisé."],
    },
    AppRule {
        packages: &["com.google.android.permissioncontroller"],
        lines: &["Permisos: dos botones y mil consecuencias. Piénsalo bien o el infierno te lo cobrará."],
    },
];

const IDLE: &[&str] = &[
    "¿Sigues ahí, recluta? Parece que te has quedado dormido en el sofá.",
    "Llevas mucho tiempo sin tocar nada. Yo ya habría limpiado tres niveles.",
    "La tele gasta menos apagada. Y tú gastarías menos tiempo si te levantaras.",
    "Estoy en guardia. Si pasa un demonio por el menú, ni te enteras.",
    "Ya lo dije: no me gusta esperar. Cuando espero, rompo cosas.",
    "Me hicieron en Rust y me dibujaron a mano. Me salió un tanque. No me quejo.",
    "Tres pulsaciones del atajo y me callo. Pero mi mirada seguirá sobre ti.",
    "¿Y si apagas la tele y haces algo útil? Pregunta retórica.",
    "El mando está entre los cojines. Siempre está entre los cojines. Qué desastre de soldado.",
    "Silencio. En mi mundo es la calma antes de que algo muy grande te arranque la cabeza.",
    "Este salón no tiene demonios. Qué decepción. Qué desperdicio de armadura.",
    "Daisy me espera en la base. Y yo, atrapado en una tele con un recluta indeciso.",
    "No soy tu amigo. No soy tu mascota. Soy un arma con casco. Acuérdate.",
    "Tienes más indecisión que un Pain Elemental. Y mira que esos son difíciles de convencer.",
    "El Icono del Pecado cayó en un par de minutos. Tú aún no has elegido qué ver.",
    "Un consejo gratis: deja de buscar el sentido de la vida en la tele. No lo encontrarás.",
    "Si me preguntas, el mejor programa es el que se apaga solo. Hazme caso.",
    "Siglos en un sarcófago y aun así tengo más paciencia que tú con un menú.",
    "Cuento los fotogramas como cuento los demonios caídos. Voy por el cuatro mil.",
    "Hayden decía que tenía un plan. Yo tengo dos escopetas y ningún plan. Funciona.",
];
const NIGHT: &[&str] = &[
    "Las tantas de la noche. Los demonios también duermen, por si te lo preguntabas. Tú no.",
    "A estas horas solo estamos tú, yo y el insomnio. Dos de tres somos peligrosos.",
    "Ve a dormir, recluta. El Slayer vigila el router.",
    "Duerme. Mañana habrá más tele y más excusas.",
];
const MORNING: &[&str] = &[
    "Buenos días, recluta. El infierno no descansa, así que tú tampoco.",
    "Despierta. Hay un mundo allá afuera esperando que lo arruines.",
    "Mañana temprano y ya con la tele. Hay quien conquista planetas y tú conquistas el sofá.",
];
const EVENING: &[&str] = &[
    "Ya anochece. Es la hora en la que los demonios más gordos salen a jugar.",
    "Tarde‑noche: la hora del «un capítulo más» y del arrepentimiento.",
    "Se acabó el día. Tú no hiciste nada y yo tampoco. Pero yo estoy en una tele, tengo excusa.",
];

/// Frases de la rutina de aburrimiento, por tipo de actividad (`text` = «act:<tipo>»).
fn activity_lines(kind: &str) -> &'static [&'static str] {
    match kind {
        "bored" => &[
            "Nadie toca el mando. Me aburro. Cuando me aburro, algo acaba roto.",
            "Dos minutos sin mando y ya me pica la motosierra.",
            "El silencio es sospechoso. Voy a dar una vuelta y a mirar si hay algo que matar.",
            "Tú te quedas ahí sin hacer nada. Yo salgo a patrullar. Qué pareja tan desigual.",
            "Hora de entrenar. Un Slayer ocioso es un Slayer oxidado.",
        ],
        "walk" => &[
            "Patrullo. No he visto demonios. Esto me enfurece.",
            "Reconocimiento de la zona: un televisor y un sofá inútil.",
            "Camino de un lado a otro. Es mi manera de no matar el aburrimiento a tiros.",
            "Exploro el nivel. Encuentro píxeles muertos. Ni un solo secreto.",
            "Paseo por la pantalla con la armadura puesta. Me da igual que no se vea bien.",
        ],
        "dance" => &[
            "Bailo. Si lo cuentas, destruyo el mando.",
            "Esto no es un baile: es un entrenamiento táctico. No te rías.",
            "Un pasito. Otro. Y ya me siento un poco menos fuera de lugar.",
            "Si los Maykr me vieran bailando… Me encargaría de que no me vieran.",
            "Doy vueltas. No hay música. Mick Gordon, ¿dónde estás?",
            "Mi armadura pesa 80 kilos. Bailar con ella es el verdadero reto del infierno.",
        ],
        "point" => &[
            "Objetivo marcado: «{t}». Dispara, recluta.",
            "«{t}». Ahí lo tienes, enfocado. ¿Esperas que lo seleccione yo?",
            "Ese de ahí, «{t}». El único que brilla. Pulsa OK de una vez.",
            "Te señalo «{t}» porque tú no te decides. Qué pena de recluta.",
            "Mira «{t}». Si lo quisieras de verdad, ya lo habrías abierto.",
        ],
        "wave" => &[
            "¿Hola? ¿Hay alguien? Qué vergüenza de relevo.",
            "Saludo al salón vacío. Responde el silencio. Me gusta más que tú.",
            "Si alguien me ve, que toque el mando. O que se largue.",
            "Hola, hola. Nadie. Perfecto. Así no tengo que dar explicaciones.",
        ],
        "rest" => &[
            "Descanso un momento. No te acostumbres.",
            "Me siento. La armadura pesa. Y tú, ahí, sin hacer nada.",
            "Cinco segundos de reposo. Ni uno más. Hell no espera.",
        ],
        "look" => &[
            "Reviso el menú. Sigue igual de inútil.",
            "Observo la pantalla en busca de amenazas. Ninguna. Qué decepción.",
            "Vigilo. Siempre vigilo. Aunque no haya nada que vigilar.",
        ],
        "stretch" => &[
            "Estiro. La armadura cruje como huesos de Imp.",
            "Calentando, que esto va a ser un número de circo con casco.",
            "Un estirón. Soy un tanque, no una piedra.",
        ],
        "tired" => &[
            "Me retiro a la base. Despiértame si hay un jefe final.",
            "Se acabó la ronda. Cero demonios. Cero diversión.",
            "A mi esquina. Daisy me espera.",
            "Fin del espectáculo. Nadie aplaudió. Mejor, así no tengo que matar a nadie.",
        ],
        "back" => &[
            "Volviste, recluta. Pensé que te había abducido un portal.",
            "Mira quién tuvo la decencia de regresar. Casi me emociono. Casi.",
            "Justo cuando empezaba a limpiar la zona. Qué oportuno.",
            "Bienvenido de vuelta. Todo seguía igual: aburrido, vacío y sin ti.",
            "Ah, sigues vivo. Qué lástima, ya me había acostumbrado al silencio.",
            "No pregunto dónde estabas. No me interesa. Sigamos.",
        ],
        "abandoned" => &[
            "Llevas siglos sin tocar nada. ¿Te quedaste dormido o te mató un demonio de verdad?",
            "La tele sigue encendida y tú no estás. Qué desperdicio de energía Argent.",
            "Si estás vivo, haz una señal. Si no, apagaré la tele por ti.",
            "Esto ya es abandono de puesto. En la UAC te habrían degradado.",
            "Ni los demonios son tan lentos como tú. Y mira que un Mancubus no corre.",
            "Mueve algo, recluta. Un dedo. Una ceja. Lo que sea.",
        ],
        _ => &[],
    }
}
