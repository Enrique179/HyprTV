# Las funciones `external` de Native las busca JNI por nombre: que R8 no las renombre.
-keep class com.hyprtv.Native { *; }
# El sistema crea el servicio y las actividades por nombre desde el manifiesto (AGP ya lo conserva, esto es por si acaso).
-keep class com.hyprtv.ScreenService
