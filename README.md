# Explorador del Sistema Solar 3D

Un simulador espacial interactivo desarrollado en Rust que te permite explorar nuestro sistema solar pilotando tu propia nave espacial en vista de tercera persona.

## Video Demo

[![Ver Video](https://img.shields.io/badge/Ver_Video_en_Google_Drive-blue?style=for-the-badge&logo=google-drive)](https://drive.google.com/file/d/19RnCJQB9iwkOtanJz7qWxI9qJJf-0o3b/view?usp=sharing)

## Características Principales

###  Sistema Solar Completo
- **8 planetas realistas** con shaders procedurales únicos
- **Sol animado** con manchas solares y corona
- **Luna terrestre** con mares lunares característicos
- **Órbitas planetarias** visualizables y precisas
- **Rotaciones planetarias** auténticas

### Nave Espacial Navegable
- **Vista de tercera persona** inmersiva
- **Física de vuelo realista** con inercia y fricción
- **Controles intuitivos** estilo simulador espacial
- **Modelo 3D personalizable** (soporte para archivos .obj)
- **Cámara dinámica** que sigue automáticamente la nave

###  Skybox Estelar
- **Campo de estrellas procedural** con 800+ estrellas
- **Tres tipos de estrellas**: normales, jóvenes (azules), viejas (rojas)
- **Diferentes brillos y tamaños** para mayor realismo
- **Rotación dinámica** del skybox con la cámara

### Shaders Planetarios Avanzados
- **Tierra**: Continentes, océanos, casquetes polares, nubes animadas
- **Marte**: Óxido de hierro, casquetes polares, Valles Marineris
- **Sol**: Superficie ardiente, manchas solares, corona brillante
- **Gigantes gaseosos**: Bandas atmosféricas, tormentas, Gran Mancha Roja
- **Mercurio**: Superficie craterizada y tonos metálicos
- **Luna**: Mares lunares y cráteres realistas
- **Neptuno**: Atmósfera profunda con metano cristalizado

##  Controles

### Navegación de la Nave
- **WASD**: Movimiento direccional
  - `W` - Impulso hacia adelante
  - `S` - Impulso hacia atrás  
  - `A` - Impulso lateral izquierdo
  - `D` - Impulso lateral derecho
- **Flechas**: Rotación de la nave
  - `↑/↓` - Pitch (cabeceo)
  - `←/→` - Yaw (guiñada)
- **Q/E**: Roll (alabeo)
- **Shift**: Turbo (multiplicador de velocidad)

### Controles del Sistema
- **Space**: Pausar/Reanudar simulación
- **O**: Mostrar/Ocultar órbitas planetarias
- **+/-**: Ajustar velocidad del tiempo
- **ESC**: Salir del programa

## Tecnologías Utilizadas

- **Rust** - Lenguaje de programación principal
- **nalgebra-glm** - Matemáticas 3D y transformaciones
- **minifb** - Gestión de ventanas y entrada de usuario
- **Software Renderer** - Engine gráfico 3D desarrollado desde cero

### Componentes del Engine
- **Vertex/Fragment Shaders** personalizados
- **Z-Buffer** para ordenamiento de profundidad
- **Rasterización** optimizada de triángulos
- **Sistema de materiales** procedurales
- **Culling** y optimizaciones de renderizado

## Instalación

### Prerrequisitos
- Rust 1.70+ instalado
- Cargo (incluido con Rust)

### Clonar e Instalar
```bash
git clone https://github.com/Albu231311/Proyecto3_space_travel.git
cd Proyecto3_space_travel
cargo build --release
```

### Ejecutar
```bash
cargo run --release
```

## Estructura del Proyecto

```
src/
├── main.rs              # Punto de entrada principal
├── camera.rs            # Sistema de cámara 3D
├── color.rs             # Manejo de colores y paletas
├── fragment.rs          # Fragmentos para rasterización
├── framebuffer.rs       # Buffer de pantalla y Z-buffer
├── noise.rs             # Generadores de ruido procedural
├── obj.rs               # Cargador de modelos 3D (.obj)
├── planet_shaders.rs    # Shaders específicos de planetas
├── shaders.rs           # Sistema de vertex shaders
├── triangle.rs          # Rasterización de triángulos
└── vertex.rs            # Estructura de vértices 3D

assets/
└── nave2.obj            # Modelo 3D de la nave hecha en blender
```

## Características Técnicas

### Renderizado
- **Rasterización por software** completamente personalizada
- **Barycentric coordinates** para interpolación de fragmentos
- **Perspective-correct interpolation** de normales y coordenadas
- **Backface culling** y **frustum culling** para optimización

### Física
- **Simulación orbital** precisa con velocidades reales
- **Sistema de partículas** para la nave espacial
- **Damping** y fricción realistas
- **Transformaciones 3D** con matrices de rotación

