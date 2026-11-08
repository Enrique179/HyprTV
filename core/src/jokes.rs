//! Los chistes de la mascota. Todo offline y sin red: reglas de palabras clave sobre el paquete en primer plano y el
//! texto de la pantalla (título del elemento enfocado, etc.) → una frase en español. Tono sarcástico, de quien vive
//! dentro de una tele, sabe que es un dibujo y se aburre. Nunca se ensaña con el usuario: el sarcasmo es de cariño.

use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trigger {
    /// Cambió la app o la pantalla.
    Window = 0,
    /// El foco lleva un rato quieto en un elemento con texto.
    Focus = 1,
    /// Pasó un rato sin hacer nada.
    Idle = 2,
    /// Prueba desde los ajustes: se salta límites y horas de silencio.
    Manual = 3,
    /// Algo que la mascota hace por su cuenta durante la rutina de aburrimiento (el tipo va en `text`: «act:dance»).
    Activity = 4,
}

impl Trigger {
    pub fn from_i32(n: i32) -> Trigger {
        match n {
            0 => Trigger::Window,
            1 => Trigger::Focus,
            2 => Trigger::Idle,
            4 => Trigger::Activity,
            _ => Trigger::Manual,
        }
    }
}

pub struct Policy {
    /// 0 = callada … 3 = charlatana.
    pub chatter: i32,
    pub quiet: bool,
    pub quiet_start: i32,
    pub quiet_end: i32,
    pub muted: Vec<String>,
    /// Habla el Slayer (`pet.look=slayer`): otro juego de frases, mismas reglas de pausa y silencio.
    pub slayer: bool,
}

impl Policy {
    fn cooldown_ms(&self) -> u64 {
        match self.chatter {
            1 => 120_000,
            2 => 60_000,
            _ => 30_000,
        }
    }

    pub fn is_quiet(&self, hour: i32) -> bool {
        if !self.quiet || self.quiet_start == self.quiet_end {
            return false;
        }
        if self.quiet_start < self.quiet_end {
            (self.quiet_start..self.quiet_end).contains(&hour)
        } else {
            hour >= self.quiet_start || hour < self.quiet_end
        }
    }
}

/// Minúsculas y sin acentos, para comparar palabras clave sin preocuparse de cómo vengan escritas.
pub fn normalize(s: &str) -> String {
    s.chars()
        .flat_map(|c| c.to_lowercase())
        .map(|c| match c {
            'á' | 'à' | 'ä' => 'a',
            'é' | 'è' | 'ë' => 'e',
            'í' | 'ì' | 'ï' => 'i',
            'ó' | 'ò' | 'ö' => 'o',
            'ú' | 'ù' | 'ü' => 'u',
            'ñ' => 'n',
            other => other,
        })
        .collect()
}

/// Recorta un título para meterlo en una frase.
pub fn short(text: &str) -> String {
    let t = text.trim();
    if t.chars().count() <= 28 {
        t.to_string()
    } else {
        let cut: String = t.chars().take(27).collect();
        format!("{}…", cut.trim_end())
    }
}

pub(crate) struct TextRule {
    /// Palabras clave sin acentos; con espacios a los lados exigen palabra entera.
    pub keys: &'static [&'static str],
    pub lines: &'static [&'static str],
}

pub(crate) struct AppRule {
    pub packages: &'static [&'static str],
    pub lines: &'static [&'static str],
}

/// Todo lo que una mascota puede decir. La sarcástica de siempre y el Slayer comparten la lógica de `Jokes` y solo cambian de voz.
pub(crate) struct Voice {
    pub text: &'static [TextRule],
    pub apps: &'static [AppRule],
    pub idle: &'static [&'static str],
    pub night: &'static [&'static str],
    pub morning: &'static [&'static str],
    pub evening: &'static [&'static str],
    pub activity: fn(&str) -> &'static [&'static str],
}

pub(crate) const VOICE_SARCASTIC: Voice =
    Voice { text: TEXT_RULES, apps: APP_RULES, idle: IDLE, night: NIGHT, morning: MORNING, evening: EVENING, activity: activity_lines };

fn voice_of(policy: &Policy) -> &'static Voice {
    if policy.slayer {
        &crate::slayer::VOICE_SLAYER
    } else {
        &VOICE_SARCASTIC
    }
}

const TEXT_RULES: &[TextRule] = &[
    TextRule {
        keys: &["wi-fi", "wifi", "red inalambrica", " red "],
        lines: &[
            "El Wi‑Fi: la razón de la mitad de las discusiones en casa.",
            "Si va lento, culpa al router. A mí ni me mires, yo peso poco.",
            "Mira las barras de señal. Yo también las miro cuando me aburro, que es siempre.",
            "Internet va lento… qué novedad. Seguro que es culpa del vecino.",
            "«Conectado, sin Internet». Lo más cercano a un chiste que ha hecho tu router.",
        ],
    },
    TextRule {
        keys: &["almacenamiento", "storage", "espacio"],
        lines: &[
            "El almacenamiento está justito. Y eso que ya quité la basura.",
            "Cuatro gigas de tele. Cada app que instalas me aprieta un poco más.",
            "Ordenar el almacenamiento: el ejercicio que todos postergamos para siempre.",
            "Se ve que alguien guarda cosas «por si acaso». Spoiler: nunca las usa.",
        ],
    },
    TextRule {
        keys: &["actualizacion", "actualizar", "update"],
        lines: &[
            "Actualizaciones: prometen mejoras y te cambian los botones de sitio.",
            "¿Actualizar? Que me avise con tiempo, necesito mentalizarme.",
            "Cada actualización es una ruleta: o va mejor o se rompe algo. Apuesta con cuidado.",
            "«Nueva versión disponible». Traducción: «hemos movido todo, buena suerte».",
        ],
    },
    TextRule {
        keys: &["bluetooth", "emparejar", "mando a distancia"],
        lines: &[
            "Bluetooth: se empareja al tercer intento. Siempre.",
            "Yo emparejo al primer intento. Es lo bueno de no tener Bluetooth.",
            "Emparejar dispositivos: la parte romántica de la tecnología.",
        ],
    },
    TextRule {
        keys: &["desarrollador", "depuracion", "adb"],
        lines: &[
            "Depuración activa: el hacker de la casa eres tú. Qué orgullo.",
            "Opciones de desarrollador… así fue como llegué yo aquí. No me quejo. Mucho.",
            "Ten cuidado ahí dentro. Yo vivo de las cosas que tocaste ahí.",
        ],
    },
    TextRule {
        keys: &["privacidad", "anuncios", "publicidad", "personalizados"],
        lines: &[
            "Anuncios personalizados desactivados. Mi parte favorita de la tele.",
            "Privacidad: aquí no se manda nada a nadie. Yo ni tengo Internet. Presumo.",
            "Publicidad personalizada: te espían para venderte lo que ya compraste.",
        ],
    },
    TextRule {
        keys: &["accesibilidad"],
        lines: &[
            "Accesibilidad: aquí vivo yo. Puedo ver la pantalla, pero soy buena, lo prometo.",
            "Este menú es mi casa. Pase, pase, no mire el desorden.",
        ],
    },
    TextRule {
        keys: &["sonido", "audio", "volumen", "altavoz"],
        lines: &[
            "Más volumen no arregla los diálogos. Pero ayuda al vecino a opinar.",
            "El sonido de la tele: o susurran o explotan. Nunca hay término medio.",
            "Ecualizador de la tele: tres ajustes y ninguno suena bien.",
        ],
    },
    TextRule {
        keys: &["imagen", "brillo", "contraste", "pantalla"],
        lines: &[
            "Más brillo no arregla las películas oscuras. O sí, no sé, yo solo soy un dibujo.",
            "El modo cine existe. Casi nadie lo usa. Casi.",
            "Modo «vívido»: para ver el mundo como lo ve un caramelo.",
        ],
    },
    TextRule {
        keys: &["contrasena", "password", "iniciar sesion", "inicia sesion", "correo", "email"],
        lines: &[
            "Escribir una contraseña con el mando: el verdadero castigo del siglo.",
            "Letra por letra, con el D‑pad. Paciencia, campeón.",
            "Y la contraseña tenía mayúscula, número y un símbolo. Que Dios te ampare.",
        ],
    },
    TextRule {
        keys: &["buscar", "busqueda", "search"],
        lines: &[
            "Buscar con el mando es un deporte olímpico. Letra por letra.",
            "¿Qué buscamos? Sea lo que sea, acabaremos viendo otra cosa.",
            "Usa la voz, dicen. Yo ya probé: la tele entiende «Netflix» y poco más.",
        ],
    },
    TextRule {
        keys: &["error", "no se pudo", "sin conexion", "reintentar", "fallo", "problema"],
        lines: &[
            "Error. ¿Lo apagaste y lo volviste a prender? Lo digo en serio.",
            "Eso se rompió. Yo no fui, estaba dormida. Tengo coartada.",
            "«Algo salió mal». Qué específico. Qué útil. Qué ganas de reírme.",
        ],
    },
    TextRule {
        keys: &["cargando", "loading", "espera"],
        lines: &[
            "Cargando… yo también me siento así los lunes.",
            "Esa ruedita y yo hemos pasado mucho tiempo juntas.",
            "Cargando. Es el único momento de la tele en que se oye pensar.",
        ],
    },
    TextRule {
        keys: &["anuncio", " ad ", "patrocinado", "saltar anuncio"],
        lines: &[
            "Un anuncio. De los que yo quité de tu inicio. De nada.",
            "Cinco segundos y lo saltas, ¿no? Yo también los cuento.",
            "Publicidad: la razón por la que existe el botón de silenciar.",
        ],
    },
    TextRule {
        keys: &["noticias", "informativo", "telediario", "ultima hora"],
        lines: &[
            "Noticias: hoy también pasó de todo. Me entero mañana.",
            "Última hora: el mando sigue sin pilas en el sofá.",
            "Las noticias de siempre con música dramática nueva.",
        ],
    },
    TextRule {
        keys: &["trailer", "avance"],
        lines: &[
            "Un tráiler de dos minutos que ya cuenta toda la peli. Clásico.",
            "«{t}»: seguro que lo mejor ya salió en el tráiler.",
            "Tráiler: el mejor minuto de la película. El resto, relleno.",
        ],
    },
    TextRule {
        keys: &["tutorial", "como hacer", "como arreglar", "paso a paso"],
        lines: &[
            "Tutorial: diez minutos de intro para decir «enchúfalo».",
            "«{t}»… y luego sobra un tornillo. Siempre sobra un tornillo.",
            "«Es muy fácil», dice el que lleva años haciéndolo.",
        ],
    },
    TextRule {
        keys: &["receta", "cocina", "cocinar", "postre"],
        lines: &[
            "Receta en la tele: te da hambre y la nevera está muy lejos.",
            "«{t}». Yo no como, pero opino: se ve riquísimo y tú no lo vas a hacer.",
            "Ver recetas es cocinar para valientes: sin ensuciar nada.",
        ],
    },
    TextRule {
        keys: &["futbol", " gol ", "liga ", "partido", "mundial", "champions", "deportes"],
        lines: &[
            "¡Gooool! …o eso gritó el vecino. ¿Cómo va el marcador?",
            "Deporte en la tele: la comodidad de ganar sin sudar.",
            "«{t}»: los fines de semana este sofá es una grada.",
            "El árbitro tuvo la culpa. Siempre la tiene. Es su trabajo.",
        ],
    },
    TextRule {
        keys: &["baloncesto", "nba", "tenis", "formula 1", "ufc", "boxeo", "rugby", "beisbol"],
        lines: &[
            "Deportes que no practicas, comentados por ti. Un clásico del sofá.",
            "«{t}». Opinas como un entrenador y te cansas subiendo la escalera.",
        ],
    },
    TextRule {
        keys: &["infantil", "dibujos", "kids", "ninos", "peppa", "bluey"],
        lines: &[
            "Dibujos animados. Entre dibujos nos entendemos.",
            "Ese tema musical se te va a quedar en la cabeza todo el día. De nada.",
            "Los niños ven dibujos. Tú también. Nadie dice nada.",
        ],
    },
    TextRule {
        keys: &["terror", "miedo", "horror", "paranormal"],
        lines: &[
            "Terror. Yo me escondo detrás del menú, avísame cuando acabe.",
            "Apaga la luz si te atreves. Yo ya cerré los ojos.",
            "Un fantasma en la casa. Pues a mí que no me mire, que solo soy píxeles.",
        ],
    },
    TextRule {
        keys: &["documental", "naturaleza", "historia de"],
        lines: &[
            "Un documental: aprendes mucho y a los diez minutos duermes.",
            "«{t}». Me encanta que la tele aún enseñe cosas. Cuando no la usan para otra cosa.",
        ],
    },
    TextRule {
        keys: &["capitulo", "episodio", "temporada", "serie"],
        lines: &[
            "Un capítulo más y duermes. Lo dijiste hace tres capítulos.",
            "«{t}»: maratón a la vista. Yo traigo las palomitas de píxeles.",
            "Temporada completa en una noche: así se hace. Mañana, a trabajar con ojeras.",
        ],
    },
    TextRule {
        keys: &["pelicula", "movie", "estreno", "cine"],
        lines: &[
            "Veinte minutos eligiendo película, cinco viéndola. Matemática pura.",
            "«{t}»: la peli que prometiste ver «algún día». Hoy tampoco.",
            "Elegir película: el verdadero reto del fin de semana.",
        ],
    },
    TextRule {
        keys: &["en vivo", " live ", "directo"],
        lines: &[
            "En directo: si te lo pierdes, ya no hay repetición. ¡Qué emoción!",
            "«{t}», en vivo. Sin trampa ni cartón. Bueno, con muchos anuncios.",
        ],
    },
    TextRule {
        keys: &["gameplay", "videojuego", "gaming", "minecraft", "fortnite", "roblox"],
        lines: &[
            "Ver a otro jugar para no jugar tú: eficiencia máxima.",
            "«{t}». Yo de joven fui pixel en un juego. Un tema delicado.",
            "Mirar cómo juegan otros: la forma más barata de tener nivel.",
        ],
    },
    TextRule {
        keys: &["musica", "lofi", "playlist", "concierto", "reggaeton", "karaoke", " mix "],
        lines: &[
            "«{t}»: pon música, que yo bailo. Mientras no lo estés viendo.",
            "Música en la tele: el sonido ya sale mejor que mi baile.",
            "Karaoke de sofá: el que no canta, no vive. El que canta, vive mal.",
        ],
    },
    TextRule {
        keys: &["reaccion", "reaction", "react"],
        lines: &[
            "Ver a alguien ver algo. Meta‑ver: nivel Dios.",
            "«{t}»: la reacción de la reacción de la reacción…",
        ],
    },
    TextRule {
        keys: &["suscribete", "suscribirse", "premium", "suscripcion", "plan mensual", "pruebalo gratis"],
        lines: &[
            "Suscripciones: pagas diez servicios para ver una serie. Matemática inversa.",
            "«Pruébalo gratis»… hasta que te acuerdes de cancelarlo. Spoiler: no te acuerdas.",
            "Premium: lo mismo, pero sin anuncios y por un precio que cambia cada tres meses.",
        ],
    },
    TextRule {
        keys: &["omitir", "saltar intro", "saltar resumen", "skip"],
        lines: &[
            "Omitir intro: el botón más usado de la historia de la televisión.",
            "Sí, sáltalo. Nadie se ha leído jamás los créditos de principio a fin.",
        ],
    },
    TextRule {
        keys: &["continuar viendo", "reanudar", "seguir viendo", "retomar"],
        lines: &[
            "«Continuar viendo»: la lista de cosas que dejaste a medias. Es larga.",
            "Retomar lo que empezaste. Qué responsable. Nunca lo habría dicho.",
        ],
    },
    TextRule {
        keys: &["favoritos", "mi lista", "guardados", "ver mas tarde"],
        lines: &[
            "«Mi lista»: el cementerio de las cosas que ibas a ver.",
            "Guardado para después. El «después» no tiene fecha. Lo sé de buena fuente: yo.",
        ],
    },
    TextRule {
        keys: &["historial", "recientes", "visto recientemente"],
        lines: &[
            "El historial: la prueba de lo que viste. Yo no diré nada. Mucho.",
            "Revisando el historial… sin juicios. Pocos. Muy pocos.",
        ],
    },
    TextRule {
        keys: &["subtitulos", "idioma", "doblaje", "audio original"],
        lines: &[
            "Subtítulos: el esfuerzo de leer mientras ves. El cine, versión gimnasio.",
            "Original o doblado: el debate eterno. Yo sin voz, así que me da igual.",
        ],
    },
    TextRule {
        keys: &["control parental", "perfil infantil", " pin ", "bloqueo"],
        lines: &[
            "Control parental: el PIN que olvidas tú antes que tus hijos.",
            "Un PIN de cuatro dígitos. Seguro es 1234. No me mires así.",
        ],
    },
    TextRule {
        keys: &["reiniciar", "apagar", "suspender", "restablecer"],
        lines: &[
            "¿Reiniciar? Sí, la solución a todo en informática y en la vida.",
            "Si me apagas, ni me despido. Pero vuelvo, siempre vuelvo.",
            "Restablecer de fábrica: borrar todo, incluyéndome. Piénsalo con calma.",
        ],
    },
    TextRule {
        keys: &["cerrar sesion", "cuenta", "perfil", "usuario"],
        lines: &[
            "Tu cuenta: más datos tuyos de los que tus amigos conocen.",
            "¿Cerrar sesión? Luego te toca escribir la contraseña con el mando. Tú verás.",
        ],
    },
    TextRule {
        keys: &["descargar", "instalar", "desinstalar", "install"],
        lines: &[
            "Instalar una app más: la TV suspira. Yo también, pero en silencio.",
            "Desinstalar: el verdadero acto de amor a este almacenamiento.",
            "«Descargando…». Tiempo suficiente para repensar tu vida.",
        ],
    },
    TextRule {
        keys: &["permitir", "aceptar", "terminos", "condiciones", "acepto"],
        lines: &[
            "Aceptar los términos sin leerlos: la firma más usada del mundo.",
            "Cuarenta páginas de condiciones. Las leyó una persona en 2011.",
            "Permitir, permitir, permitir… y luego te extrañas de lo que saben de ti.",
        ],
    },
    TextRule {
        keys: &["notificaciones", "avisos"],
        lines: &[
            "Notificaciones: 99 % anuncios, 1 % lo que necesitabas.",
            "Otra notificación. Qué ganas tenían de hablarte.",
        ],
    },
    TextRule {
        keys: &["canales", "canal ", "guia de programacion", "tv en vivo", "zapping"],
        lines: &[
            "Cien canales y nada que ver. La paradoja de la televisión.",
            "Zapping: ver el mismo anuncio en cuatro canales distintos.",
            "«{t}». Tantos canales y siempre acabas en el mismo.",
        ],
    },
    TextRule {
        keys: &["radio", "podcast"],
        lines: &[
            "Radio en una tele: un televisor de lujo para no mirar.",
            "Podcast con la pantalla encendida: gasto de luz con ambición.",
        ],
    },
    TextRule {
        keys: &["anime", "manga", "otaku"],
        lines: &[
            "Anime: el único lugar donde una cena puede durar tres capítulos.",
            "«{t}». Gritos, flashback, otro grito y el siguiente capítulo.",
        ],
    },
    TextRule {
        keys: &["comedia", "stand up", "humor", "chiste"],
        lines: &[
            "Comedia: si la competencia es dura, hasta yo sabría. Casi.",
            "Un humorista en la tele. Lo mío es gratis y no tiene aplausos.",
        ],
    },
    TextRule {
        keys: &["viaje", "vlog", "turismo", "mochilero"],
        lines: &[
            "Viajar viendo vlogs: conoce el mundo sin salir del sofá ni gastar un euro.",
            "«{t}». Qué bonito. Yo he recorrido esquina a esquina esta pantalla, que no es poco.",
        ],
    },
    TextRule {
        keys: &["review", "unboxing", "tecnologia", "reseña", "comparativa"],
        lines: &[
            "Un unboxing: ver cómo otro abre cajas. Sí, la vida es esto.",
            "Reseñas de tecnología: te enseñan lo que no necesitas y quieres igual.",
        ],
    },
    TextRule {
        keys: &["oferta", "descuento", "gratis", "promocion", "rebajas"],
        lines: &[
            "«Oferta limitada». Nunca lo es, pero funciona.",
            "Gratis: el precio más caro del mundo.",
        ],
    },
    TextRule {
        keys: &["ranking", "los mejores", " top ", "tendencias", "popular"],
        lines: &[
            "Los más vistos: si todos lo ven, por algo será. O por nada, ya sabes cómo va.",
            "Tendencias: la lista de lo que no sabías que necesitabas ver.",
        ],
    },
    TextRule {
        keys: &["clima", "pronostico", "temperatura", "lluvia"],
        lines: &[
            "El clima: acertaron el martes pasado y desde entonces se han relajado.",
            "Pronóstico del tiempo: la única profesión donde fallar es normal.",
        ],
    },
    TextRule {
        keys: &["fondo de pantalla", "salvapantallas", "protector"],
        lines: &[
            "Un salvapantallas: porque mirar una pared apagada era muy aburrido.",
            "Fondos de pantalla: la decoración de una casa donde solo vivo yo. Gracias.",
        ],
    },
    TextRule {
        keys: &["hyprtv", "espacios de trabajo", "borde de foco", "mascota"],
        lines: &[
            "¿Me estás configurando? Qué intimidad. Con cuidado, que soy delicada.",
            "Ajustes de mí misma: lo más cerca que estaré de la terapia.",
            "Cambia lo que quieras. Yo sigo aquí. No tengo adónde ir.",
        ],
    },
];

const APP_RULES: &[AppRule] = &[
    AppRule {
        packages: &["com.google.android.youtube.tv", "com.google.android.youtube"],
        lines: &[
            "YouTube: entras por un video de tres minutos y sales a las cuatro de la mañana.",
            "El algoritmo te conoce mejor que yo. Y yo te veo a diario.",
            "YouTube: 2 mil millones de videos y siempre acabas en el de los gatitos.",
            "Si no es un tutorial de arreglar la tele, no me interesa. Es mentira, sí me interesa.",
            "Abriste YouTube «un momento». Cuento los minutos, no te preocupes.",
            "Recomendado para ti: otra vez lo mismo de siempre, con otra miniatura.",
        ],
    },
    AppRule {
        packages: &["com.android.tv.settings", "com.android.settings", "com.google.android.tvlauncher.settings"],
        lines: &[
            "Ajustes: entras a cambiar una cosa y sales con doce.",
            "Qué serios son los ajustes. Que alguien les cuente un chiste.",
            "No toques nada raro, que yo vivo aquí.",
            "Ajustes de la tele. Aquí se decide el destino del brillo y poco más.",
            "Mirando ajustes en vez de ver algo. Productividad pura.",
        ],
    },
    AppRule {
        packages: &["com.android.vending"],
        lines: &[
            "Play Store: «solo voy a mirar» es una mentira muy bonita.",
            "Instalar, instalar, instalar… y el almacenamiento llorando en una esquina.",
            "Tienda de apps. Cada app nueva es un adiós a un poquito de RAM.",
            "Otra app más para la colección que no abres. Qué ilusión.",
            "Ojo con instalar: lo que entra a esta tele no sale. Bueno, sí. Pero cuesta.",
        ],
    },
    AppRule {
        packages: &["com.android.mgstv"],
        lines: &[
            "Tantos canales y siempre acabas viendo el mismo. Lo sabemos.",
            "Hora de zapping. El deporte favorito de este sofá.",
            "Mucha oferta, poco que ver. La televisión de siempre.",
            "Xuper: cuarenta mil títulos y tú buscando «algo». Qué talento.",
            "La próxima vez di una cosa concreta. «Algo bueno» no es un género.",
        ],
    },
    AppRule {
        packages: &["com.instantbits.cast.receiver"],
        lines: &[
            "El teléfono manda y la tele obedece. Como en casa.",
            "Enviar videos a la tele desde el móvil: pequeña magia moderna.",
            "Web Video Cast: lo que tu móvil quiere ver, lo ves tú. Qué generosa es la tecnología.",
        ],
    },
    AppRule {
        packages: &["com.google.android.katniss", "com.google.android.googlequicksearchbox"],
        lines: &[
            "¿Hablándole a la tele? Yo hago lo mismo, y nadie me contesta.",
            "Búsqueda por voz: dices «película de acción» y te ponen un documental de abejas.",
        ],
    },
    AppRule {
        packages: &["com.hyprtv", "com.google.android.apps.tv.launcherx", "com.google.android.tvlauncher"],
        lines: &[
            "Bienvenido al escritorio. Aquí no hay papelera, pero sí mucho estilo.",
            "Muchas apps, y acabas en YouTube. Siempre acabas en YouTube.",
            "Este launcher pesa poco. Llevo una dieta estricta.",
            "Estoy hecha en Rust. Por eso gasto tan poco. Y por eso presumo tanto.",
            "Tres pulsaciones del atajo y me voy. No me ofendo. Mucho.",
            "Inicio. Donde empiezan todos los planes de «hoy sí hago algo productivo».",
            "Volviste al inicio. Como cuando uno abre la nevera sin saber qué buscaba.",
        ],
    },
    AppRule {
        packages: &["com.google.android.apps.mediashell", "com.google.android.gms"],
        lines: &["Proyectando algo desde el móvil. Como si la tele no tuviera nada que decir."],
    },
    AppRule {
        packages: &["com.google.android.packageinstaller", "com.android.packageinstaller"],
        lines: &["Instalando algo. Respira hondo: es una app, no una boda."],
    },
    AppRule {
        packages: &["com.google.android.permissioncontroller"],
        lines: &["Permisos: la parte donde una app te pide la luna y tú le das la mitad."],
    },
];

const IDLE: &[&str] = &[
    "¿Sigues ahí? Yo sí. No tengo a dónde ir.",
    "Sé que estás mirando la pantalla. Te veo mirándome.",
    "Recuerda parpadear: la pantalla se come los ojos.",
    "Un dato: la tele gasta menos apagada. Solo lo digo.",
    "Tengo 1,5 GB de RAM para compartir con YouTube. Qué convivencia.",
    "Me programaron en Rust y me dibujaron con curvas de Bézier. Vida de lujo.",
    "Si quieres que me calle, tres pulsaciones del atajo y me duermo. Y no me ofendo. Casi.",
    "¿Y si apagamos la tele y vamos a la nevera? Es solo una idea.",
    "El mando está más cerca de lo que crees. Debajo del cojín, seguro.",
    "Yo vivo en un televisor. Tú, en un sofá. Y aun así soy yo la que no se mueve.",
    "Esto de ser una mascota sin mascota. Qué crisis existencial tan bien iluminada.",
    "Cada vez que no tocas nada, un píxel envejece. No es verdad, pero queda dramático.",
    "Qué silencio. Mi hobby favorito después del silencio es el silencio.",
    "Dato inútil: la pantalla tiene millones de píxeles. Yo ocupo unos mil. Qué modesta soy.",
    "Me aburro tanto que ya cuento los fotogramas. Voy por el cuatro mil.",
    "Si fuera un humano me pondría a ordenar. Soy un dibujo: me conformo con mirar.",
    "Aquí estoy, esperando órdenes. Y comentarios sarcásticos. Principalmente eso.",
    "Qué tentación apagar el Wi‑Fi para ver qué haces. Es broma. No tengo ese permiso.",
    "Apuesto a que estás pensando en algo que ver. Y no vas a ver nada de eso. Todos igual.",
    "No es que me importe lo que haces. Es que no tengo otra cosa que mirar.",
    "Si pestañeo, ¿cuenta como ver la tele? Pregunto por un amigo que soy yo.",
    "Mi mayor logro hasta hoy: no ocupar más de 25 megas. Léelo con orgullo.",
    "Tú pulsa botones, que yo me encargo de comentar lo inútiles que son.",
    "Aquí donde me ves, soy más ligera que el anuncio que acabas de saltarte.",
    "Un consejo gratis: no busques el sentido de la vida en la tele. Está en el otro canal.",
    "Estoy tan quieta que me empiezan a salir ideas. Y dan miedo.",
    "Si me preguntas, la mejor serie de la tele es cuando se queda en negro.",
    "No te lo tomes a mal, pero llevas rato mirando el menú. Hay más cosas en la vida. Dos o tres.",
];
const NIGHT: &[&str] = &[
    "¿Y tú no duermes? Yo sí, a ratos.",
    "Qué hora para estar despierto… me gusta.",
    "Las tantas de la noche. Es el momento en que la tele sabe más que tú.",
    "A estas horas solo hay dos tipos de personas: los que trabajan y los que no pueden dormir. Adivina.",
];
const MORNING: &[&str] = &[
    "Buenos días. Todavía no me despierto del todo.",
    "Un café y un video, y estamos listos.",
    "Mañana temprano: la tele es lo único que funciona antes que tú.",
];
const EVENING: &[&str] = &[
    "Ya es de noche. Perfecto para algo lento y con manta.",
    "El día se acabó; la lista de series, no.",
    "Tarde‑noche: hora de decir «un capítulo más» y no cumplirlo.",
];

/// Frases de la rutina de aburrimiento, por tipo de actividad (`text` = «act:<tipo>»).
fn activity_lines(kind: &str) -> &'static [&'static str] {
    match kind {
        "bored" => &[
            "Veo que me dejaste sola. Qué novedad.",
            "Pasaron los minutos y nadie toca nada. Voy a hacer algo productivo: nada, pero con estilo.",
            "Tele encendida, nadie mirando. Un clásico del ahorro de energía.",
            "Se acabó la tranquilidad: me aburro y yo cuando me aburro hago cosas.",
            "Dos minutos sin mando. Debe de ser un récord personal tuyo.",
            "Voy a aprovechar que no miras. Digo, que no estás.",
        ],
        "walk" => &[
            "Doy una vuelta. Total, el espacio es mío.",
            "Voy a estirar las piernas. Las dos que tengo.",
            "Paseo por la pantalla: la única propiedad que tengo.",
            "Ruta turística por el televisor. Sin guía, sin mapa, sin gracia.",
            "Camino de un lado a otro. Es lo más parecido a cardio que tengo.",
        ],
        "dance" => &[
            "Bailo sin música. Cero ritmo, cero pudor.",
            "Si nadie mira, no cuenta como ridículo.",
            "Esto va para mi currículum: «bailarina de menú».",
            "Sin música, el baile es más honesto. Y más raro.",
            "Un pasito. Otro. Y ya me siento un poco menos pixel.",
            "Practico mis pasos por si algún día pones música. Que no lo harás.",
            "Esto no es un baile: es un grito de auxilio con coreografía.",
            "Doy vueltas porque no hay nada más que hacer. Ni más sitio.",
        ],
        "point" => &[
            "Mira ese: «{t}». Impresionante. Digno de estatua.",
            "Ahí, «{t}». Llevas rato sin tocarlo, no te culpo.",
            "«{t}». Es lo único seleccionado y aun así no te decides.",
            "Ese de ahí. El que está resaltado. Increíble lo que me entretiene.",
            "Señalo lo que tienes enfocado. Tú no lo harás, así que lo hago yo.",
        ],
        "wave" => &[
            "¿Hola? ¿Alguien por ahí? Nada.",
            "Saludo al vacío. Me devuelve el saludo, claro: silencio.",
            "Si alguien me ve, que haga una señal. O que toque el mando, vaya.",
            "Hola, hola. No hay nadie. Perfecto, así sigo.",
        ],
        "rest" => &[
            "Me siento. El esfuerzo de existir cansa.",
            "Descanso cinco segundos. Me lo he ganado.",
            "A veces lo mejor que se puede hacer es sentarse a ver cómo pasa el tiempo.",
        ],
        "look" => &[
            "Leo el menú. Sigue siendo igual de emocionante.",
            "Miro de un lado a otro. Esto es lo que se llama «disfrutar del contenido».",
            "Examino la pantalla en busca de algo interesante. Resultado: nada.",
        ],
        "stretch" => &[
            "Estiro. Mis articulaciones son líneas, así que ni crujen.",
            "Un bostezo para dar ambiente.",
            "Calentando, que esto va a ser un número de circo.",
        ],
        "tired" => &[
            "Me canso. Me duermo. Despiértenme cuando haya algo que ver. O nunca.",
            "Buenas noches, mundo cruel. Me vuelvo a mi esquina.",
            "Se acabó el espectáculo. No hubo aplausos. Típico.",
            "Me voy a dormir un rato. Ya me dirás si pasó algo emocionante. Seguro que no.",
        ],
        "back" => &[
            "¡Ah, sigues vivo! Pensé que te habías mudado.",
            "Volviste. Qué honor. Casi tengo ganas de fingir que no me aburría.",
            "Tranquilo, no hacía nada raro. Bueno, bailaba. Eso es todo.",
            "Menos mal, ya me estaba hablando sola. Otra vez.",
            "Mira quién se acordó de mí. Y del mando.",
            "Justo cuando empezaba el segundo número. Qué sentido de la oportunidad.",
            "No te pongo cara de reproche porque soy un dibujo. Pero la tengo pensada.",
        ],
        "abandoned" => &[
            "Llevas un buen rato sin tocar el mando. ¿Estás bien? Parpadea dos veces.",
            "La tele sigue encendida. La factura de la luz te manda saludos.",
            "Si te quedaste dormido, buenas noches. Si no, apaga la tele y duerme.",
            "Esto ya es abandono de tele. Voy a llamar a protección de electrodomésticos.",
            "Hace rato que no pasa nada. Si es un experimento de paciencia, vas ganando.",
            "Te lo digo con cariño: la tele no se ve sola. Aunque a veces lo parece.",
        ],
        _ => &[],
    }
}

pub struct Jokes {
    recent: VecDeque<&'static str>,
    last_spoke: Option<u64>,
    last_activity: Option<u64>,
    last_pkg: String,
    seed: u32,
}

impl Default for Jokes {
    fn default() -> Self {
        Jokes { recent: VecDeque::new(), last_spoke: None, last_activity: None, last_pkg: String::new(), seed: 0x9E37_79B9 }
    }
}

impl Jokes {
    fn rand(&mut self, n: usize) -> usize {
        let mut x = self.seed;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.seed = x;
        (x as usize) % n.max(1)
    }

    fn pick(&mut self, mut pool: Vec<&'static str>, title: &str) -> Option<String> {
        if title.is_empty() {
            pool.retain(|l| !l.contains("{t}"));
        }
        let fresh: Vec<&'static str> = pool.iter().copied().filter(|l| !self.recent.contains(l)).collect();
        let pool = if fresh.is_empty() { pool } else { fresh };
        if pool.is_empty() {
            return None;
        }
        let line = pool[self.rand(pool.len())];
        self.recent.push_back(line);
        // Más memoria que antes: hay muchas más frases y repetir pronto sería lo primero que se nota.
        while self.recent.len() > 14 {
            self.recent.pop_front();
        }
        Some(line.replace("{t}", &short(title)))
    }

    fn activity(&mut self, kind: &str, now: u64, hour: i32, title: &str, policy: &Policy) -> Option<String> {
        if policy.chatter <= 0 || policy.is_quiet(hour) {
            return None;
        }
        // El aviso de que empieza, se cansa o volvió alguien siempre se dice; el resto de movimientos, solo a veces
        // y con pausa, o hablaría sin parar.
        let essential = matches!(kind, "bored" | "tired" | "back" | "abandoned");
        if !essential {
            if let Some(t) = self.last_activity {
                if now.saturating_sub(t) < 18_000 {
                    return None;
                }
            }
            let odds = match policy.chatter {
                1 => 3,
                2 => 2,
                _ => 1,
            };
            if self.rand(odds) != 0 {
                return None;
            }
        }
        let line = self.pick((voice_of(policy).activity)(kind).to_vec(), title)?;
        self.last_activity = Some(now);
        self.last_spoke = Some(now);
        Some(line)
    }

    /// ¿Dice algo ahora? `now` en ms, `hour` la hora local 0–23, `text` lo que se ve en pantalla (o «act:<tipo>» en una
    /// actividad) y `title` el título del elemento enfocado (puede estar vacío).
    #[allow(clippy::too_many_arguments)]
    pub fn decide(&mut self, trigger: Trigger, now: u64, hour: i32, pkg: &str, text: &str, title: &str, policy: &Policy) -> Option<String> {
        self.seed ^= (now as u32).wrapping_mul(2_654_435_761) | 1;
        if trigger == Trigger::Activity {
            return self.activity(text.strip_prefix("act:").unwrap_or(text), now, hour, title, policy);
        }
        let manual = trigger == Trigger::Manual;
        if !manual {
            if policy.chatter <= 0 || policy.is_quiet(hour) || policy.muted.iter().any(|m| m == pkg) {
                return None;
            }
            if let Some(t) = self.last_spoke {
                if now.saturating_sub(t) < policy.cooldown_ms() {
                    return None;
                }
            }
            // El foco quieto solo habla con charla media o alta.
            if matches!(trigger, Trigger::Focus) && policy.chatter < 2 {
                return None;
            }
        }

        let hay = format!(" {} {} ", normalize(title), normalize(text));
        let voice = voice_of(policy);
        let mut text_pool: Vec<&'static str> = Vec::new();
        for rule in voice.text {
            if rule.keys.iter().any(|k| hay.contains(k)) {
                text_pool.extend(rule.lines.iter().copied());
            }
        }
        let app_pool: Vec<&'static str> =
            voice.apps.iter().filter(|r| r.packages.contains(&pkg)).flat_map(|r| r.lines.iter().copied()).collect();

        let line = match trigger {
            // El foco solo habla si hay algo concreto que comentar.
            Trigger::Focus => self.pick(text_pool, title),
            Trigger::Window => {
                if !manual && self.last_pkg == pkg {
                    return None;
                }
                // Una vez por cambio de app: lo concreto de la pantalla gana a lo general de la app.
                let pool = if !text_pool.is_empty() { text_pool } else { app_pool };
                self.pick(pool, title)
            }
            _ => {
                let mut pool: Vec<&'static str> = voice.idle.to_vec();
                match hour {
                    0..=5 => pool.extend(voice.night),
                    6..=11 => pool.extend(voice.morning),
                    19..=23 => pool.extend(voice.evening),
                    _ => {}
                }
                if manual {
                    pool.extend(app_pool);
                    pool.extend(text_pool);
                }
                self.pick(pool, title)
            }
        };
        if line.is_some() {
            self.last_spoke = Some(now);
            if matches!(trigger, Trigger::Window | Trigger::Manual) {
                self.last_pkg = pkg.to_string();
            }
        } else if trigger == Trigger::Window {
            // Aunque no haya dicho nada, esa app ya pasó: no insistir al volver a ella enseguida.
            self.last_pkg = pkg.to_string();
        }
        line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(chatter: i32) -> Policy {
        Policy { chatter, quiet: true, quiet_start: 23, quiet_end: 7, muted: vec![], slayer: false }
    }

    fn slayer(chatter: i32) -> Policy {
        Policy { slayer: true, ..policy(chatter) }
    }

    const KINDS: &[&str] = &["bored", "walk", "dance", "point", "wave", "rest", "look", "stretch", "tired", "back", "abandoned"];

    #[test]
    fn normalize_strips_accents() {
        assert_eq!(normalize("Película ÉPICA, año"), "pelicula epica, ano");
    }

    #[test]
    fn short_truncates_on_chars_not_bytes() {
        assert_eq!(short("corto"), "corto");
        let long = "ñandú ".repeat(10);
        let s = short(&long);
        assert!(s.chars().count() <= 28 && s.ends_with('…'));
    }

    #[test]
    fn window_change_talks_once_per_app() {
        let mut j = Jokes::default();
        let p = policy(3);
        let a = j.decide(Trigger::Window, 1_000_000, 15, "com.google.android.youtube.tv", "", "", &p);
        assert!(a.is_some());
        // Misma app y cooldown cumplido: no repite el comentario de la app.
        assert!(j.decide(Trigger::Window, 2_000_000, 15, "com.google.android.youtube.tv", "", "", &p).is_none());
        // Otra app: sí.
        assert!(j.decide(Trigger::Window, 3_000_000, 15, "com.android.vending", "", "", &p).is_some());
    }

    #[test]
    fn cooldown_quiet_hours_mute_and_silent_level() {
        let mut j = Jokes::default();
        let p = policy(2);
        assert!(j.decide(Trigger::Window, 1_000_000, 15, "com.android.vending", "", "", &p).is_some());
        assert!(j.decide(Trigger::Window, 1_010_000, 15, "com.android.tv.settings", "", "", &p).is_none(), "dentro del cooldown");
        assert!(j.decide(Trigger::Window, 2_000_000, 23, "com.android.tv.settings", "", "", &p).is_none(), "hora de silencio");
        let mut muted = policy(2);
        muted.muted.push("com.android.tv.settings".into());
        assert!(j.decide(Trigger::Window, 3_000_000, 15, "com.android.tv.settings", "", "", &muted).is_none());
        assert!(j.decide(Trigger::Window, 4_000_000, 15, "com.android.tv.settings", "", "", &policy(0)).is_none());
        // La prueba manual ignora todo eso.
        assert!(j.decide(Trigger::Manual, 4_000_001, 3, "com.android.tv.settings", "", "", &policy(0)).is_some());
    }

    #[test]
    fn focus_needs_a_keyword_and_fills_the_title() {
        let mut j = Jokes::default();
        let p = policy(3);
        assert!(j.decide(Trigger::Focus, 1_000_000, 15, "x", "", "Cosas aleatorias", &p).is_none());
        let line = j.decide(Trigger::Focus, 2_000_000, 15, "x", "", "El tráiler de la nueva película", &p).unwrap();
        assert!(!line.contains("{t}"));
        // Con poca charla el foco no habla.
        assert!(j.decide(Trigger::Focus, 9_000_000, 15, "x", "", "Wi-Fi", &policy(1)).is_none());
    }

    #[test]
    fn accents_in_the_screen_text_still_match() {
        let mut j = Jokes::default();
        let line = j.decide(Trigger::Focus, 1_000_000, 15, "x", "Almacenamiento y reinicio", "", &policy(3));
        assert!(line.is_some());
    }

    #[test]
    fn no_immediate_repeats() {
        let mut j = Jokes::default();
        let mut seen = Vec::new();
        for i in 0..8 {
            let l = j.decide(Trigger::Manual, 1_000 + i * 7, 15, "com.hyprtv", "", "", &policy(3)).unwrap();
            assert!(!seen.contains(&l), "repetida demasiado pronto: {l}");
            seen.push(l);
        }
    }

    #[test]
    fn title_placeholder_is_never_left_behind() {
        let mut j = Jokes::default();
        for i in 0..80 {
            let l = j.decide(Trigger::Manual, 1_000 + i * 13, 15, "x", "serie tutorial tráiler futbol música receta canales", "", &policy(3));
            if let Some(l) = l {
                assert!(!l.contains("{t}"), "{l}");
            }
        }
        for p in [policy(3), slayer(3)] {
            for kind in KINDS {
                let mut j = Jokes::default();
                for i in 0..30 {
                    if let Some(l) = j.decide(Trigger::Activity, 1_000_000 * (i + 1), 15, "x", &format!("act:{kind}"), "", &p) {
                        assert!(!l.contains("{t}"), "{kind}: {l}");
                    }
                }
            }
        }
    }

    #[test]
    fn quiet_hours_wrap_midnight() {
        let p = policy(2);
        assert!(p.is_quiet(23) && p.is_quiet(2) && !p.is_quiet(12) && !p.is_quiet(7));
        let day = Policy { chatter: 2, quiet: true, quiet_start: 13, quiet_end: 15, muted: vec![], slayer: false };
        assert!(day.is_quiet(14) && !day.is_quiet(15));
    }

    #[test]
    fn every_activity_kind_has_lines_and_essentials_always_speak() {
        for v in [&VOICE_SARCASTIC, &crate::slayer::VOICE_SLAYER] {
            for kind in KINDS {
                assert!(!(v.activity)(kind).is_empty(), "sin frases para {kind}");
            }
            assert!((v.activity)("no-existe").is_empty());
        }
        for p in [policy(1), slayer(1)] {
            let mut j = Jokes::default();
            for kind in ["bored", "tired", "back", "abandoned"] {
                // Tres seguidas, pegadas en el tiempo: las esenciales no se filtran ni por pausa ni por azar.
                for i in 0..3 {
                    assert!(j.decide(Trigger::Activity, 1_000 + i * 100, 15, "x", &format!("act:{kind}"), "", &p).is_some(), "{kind}");
                }
            }
        }
    }

    #[test]
    fn activity_respects_silence_and_pause() {
        let mut j = Jokes::default();
        assert!(j.decide(Trigger::Activity, 1_000, 15, "x", "act:bored", "", &policy(0)).is_none(), "charla en 0");
        assert!(j.decide(Trigger::Activity, 1_000, 2, "x", "act:bored", "", &policy(3)).is_none(), "hora de silencio");
        // Con charla máxima las no esenciales hablan siempre, pero con 18 s de pausa entre una y otra.
        let p = policy(3);
        assert!(j.decide(Trigger::Activity, 100_000, 15, "x", "act:dance", "", &p).is_some());
        assert!(j.decide(Trigger::Activity, 105_000, 15, "x", "act:walk", "", &p).is_none());
        assert!(j.decide(Trigger::Activity, 120_000, 15, "x", "act:walk", "", &p).is_some());
    }

    #[test]
    fn point_uses_the_focused_title() {
        let mut found = false;
        for seed in 0..40u64 {
            let mut j = Jokes::default();
            if let Some(l) = j.decide(Trigger::Activity, 1_000_000 + seed * 977, 15, "x", "act:point", "Spider-Man: un nuevo día", &policy(3)) {
                found |= l.contains("Spider-Man");
            }
        }
        assert!(found, "al señalar menciona el título de lo enfocado");
    }

    fn all_lines(v: &Voice) -> Vec<&'static str> {
        let mut all: Vec<&str> = v.text.iter().flat_map(|r| r.lines.iter().copied()).collect();
        all.extend(v.apps.iter().flat_map(|r| r.lines.iter().copied()));
        all.extend(v.idle.iter().copied());
        for k in KINDS {
            all.extend((v.activity)(k).iter().copied());
        }
        all
    }

    #[test]
    fn plenty_of_material() {
        for (name, v, min) in [("sarcástica", &VOICE_SARCASTIC, 250), ("slayer", &crate::slayer::VOICE_SLAYER, 150)] {
            let all = all_lines(v);
            assert!(all.len() >= min, "{name}: solo {} frases", all.len());
            // Ninguna frase se repite entre grupos (sería una errata).
            let mut sorted = all.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), all.len(), "{name}: hay frases duplicadas");
        }
    }

    #[test]
    fn each_voice_speaks_only_its_own_lines() {
        let mine = all_lines(&crate::slayer::VOICE_SLAYER);
        let theirs = all_lines(&VOICE_SARCASTIC);
        assert!(mine.iter().all(|l| !theirs.contains(l)), "frases compartidas entre voces");
        let mut j = Jokes::default();
        for i in 0..40 {
            let l = j.decide(Trigger::Manual, 1_000 + i * 31, 15, "com.android.vending", "", "", &slayer(3)).unwrap();
            assert!(mine.contains(&l.as_str()) || l.contains('«'), "el Slayer dijo algo ajeno: {l}");
        }
        let l = j.decide(Trigger::Manual, 9_000_000, 15, "com.android.vending", "", "", &policy(3)).unwrap();
        assert!(!mine.contains(&l.as_str()), "la sarcástica usó una frase del Slayer: {l}");
    }
}
