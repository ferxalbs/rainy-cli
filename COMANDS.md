# 🚀 Rainy CLI - Guía Completa de Comandos

¡Bienvenido a Rainy CLI! Esta guía te ayudará a aprovechar al máximo todas las funcionalidades de tu asistente de IA para desarrollo.

## 📋 Tabla de Contenidos

- [Comandos Básicos](#comandos-básicos)
- [Sistema de Sesiones](#sistema-de-sesiones)
- [Optimización de Tokens](#optimización-de-tokens)
- [Análisis de Código](#análisis-de-código)
- [Generación de Código](#generación-de-código)
- [Revisión de Código](#revisión-de-código)
- [Gestión de Proyectos](#gestión-de-proyectos)
- [Configuración](#configuración)
- [Mejores Prácticas](#mejores-prácticas)

---

## 🏠 Comandos Básicos

### Chat Interactivo

#### Chat Tradicional
```bash
# Chat simple (historial global)
rainy-cli chat

# Chat con mensaje inicial
rainy-cli chat "explícame cómo funciona async/await en Rust"

# Chat con archivos de contexto
rainy-cli chat --context-file src/main.rs "optimiza esta función"

# Chat con múltiples archivos de contexto
rainy-cli chat --context-file src/main.rs --context-file src/lib.rs "analiza la arquitectura"
```

#### 🆕 **Chat con Sesiones Automáticas** ✨

La funcionalidad más innovadora: **creación automática de sesiones inteligentes**

```bash
# ✨ NUEVO: Chat con sesión automática
# Se crea automáticamente una sesión con título y descripción generados por IA
rainy-cli chat "Crea un componente React de acordeón usando shadcn/ui"
rainy-cli chat "Implementa un sistema de autenticación JWT"
rainy-cli chat "¿Cómo optimizar esta consulta SQL?"

# Resultado esperado:
# 🎯 Creando sesión automática para tu consulta...
# ✅ Sesión creada: "Crea un componente React de acordeón usando sh..."
# 📝 Descripción: Conversación sobre: Crea un componente React de acordeón usando shadcn/ui
# 🆔 ID de sesión: session_1757436308542
# 💡 Puedes usar esta sesión en el futuro con: rainy-cli session chat session_1757436308542 <mensaje>
```

**¿Qué hace automáticamente?**
- 🤖 **Genera título inteligente** usando el modelo predeterminado
- 📝 **Crea descripción automática** basada en tu consulta
- 💾 **Guarda toda la conversación** en la sesión
- 🔄 **Permite continuar** la conversación en futuras sesiones
- 📊 **Optimiza tokens** al mantener contexto organizado

---

## 🎯 Sistema de Sesiones

### Gestión de Sesiones

```bash
# Crear una nueva sesión
rainy-cli session create "Proyecto Web API" --description "Desarrollo de API REST en Rust"

# Listar todas las sesiones
rainy-cli session list

# Ver detalles de una sesión específica
rainy-cli session show <session_id>

# Renombrar una sesión
rainy-cli session rename <session_id> "Nuevo Nombre"

# Actualizar descripción
rainy-cli session update-description <session_id> "Nueva descripción detallada"
```

### Etiquetas y Organización

```bash
# Agregar etiquetas a una sesión
rainy-cli session add-tag <session_id> "rust"
rainy-cli session add-tag <session_id> "api"
rainy-cli session add-tag <session_id> "backend"

# Remover etiquetas
rainy-cli session remove-tag <session_id> "deprecated"

# Buscar sesiones por nombre, descripción o etiquetas
rainy-cli session search "rust"
rainy-cli session search "api"
```

### Chat con Sesiones

```bash
# Iniciar chat con sesión existente
rainy-cli session chat <session_id>

# Chat con mensaje inicial
rainy-cli session chat <session_id> "explica el archivo main.rs"

# Chat con contexto adicional
rainy-cli session chat <session_id> "optimiza esta función" --context-file src/utils.rs

# Chat sin historial adicional (solo sesión)
rainy-cli session chat <session_id> "pregunta específica" --no-history
```

### Backup y Migración

```bash
# Exportar una sesión para backup
rainy-cli session export <session_id> backup_sesion.json

# Importar una sesión desde archivo
rainy-cli session import backup_sesion.json --name "Sesión Restaurada"

# Limpiar mensajes de una sesión (mantener estructura)
rainy-cli session clear <session_id>

# Eliminar sesión completamente
rainy-cli session delete <session_id>
```

### Ejemplos de Flujo de Trabajo

```bash
# 1. Crear sesión para proyecto específico
rainy-cli session create "E-commerce API" --description "Backend para tienda online"

# 2. Agregar etiquetas para organización
rainy-cli session add-tag <session_id> "rust"
rainy-cli session add-tag <session_id> "api"
rainy-cli session add-tag <session_id> "ecommerce"

# 3. Trabajar en la sesión
rainy-cli session chat <session_id> "diseña el esquema de base de datos"
rainy-cli session chat <session_id> "implementa los endpoints de productos"

# 4. Exportar para backup
rainy-cli session export <session_id> ecommerce_backup.json
```

---

## ⚡ Optimización de Tokens

### Para Prompts Cortos (Ahorra Tokens)

```bash
# Sin historial (máximo ahorro)
rainy-cli chat --no-history "cuál es la sintaxis de match en Rust?"
rainy-cli session chat <session_id> "sintaxis básica" --no-history

# Preguntas específicas sin contexto
rainy-cli chat --no-history "cómo instalar Rust?"
rainy-cli chat --no-history "diferencia entre Vec y Array"
```

### Para Conversaciones Continuas

```bash
# Historial optimizado automáticamente
rainy-cli chat "explícame este código complejo paso a paso"
rainy-cli session chat <session_id> "continúa explicando el patrón de diseño"

# Mantiene contexto pero ahorra tokens
rainy-cli chat "ahora muestra un ejemplo práctico"
```

### Con Contexto Específico

```bash
# Análisis de archivo específico
rainy-cli chat --context-file src/main.rs "optimiza esta función"
rainy-cli session chat <session_id> "analiza este módulo" --context-file src/utils.rs

# Múltiples archivos
rainy-cli chat --context-file src/lib.rs --context-file src/main.rs "compara estas implementaciones"
```

---

## 🔍 Análisis de Código

### Análisis General

```bash
# Análisis completo del proyecto
rainy-cli analyze -p . -a general

# Análisis de archivo específico
rainy-cli analyze -p src/main.rs -a general

# Análisis de directorio
rainy-cli analyze -p src/ -a general
```

### Análisis Especializado

```bash
# Seguridad
rainy-cli analyze -p . -a security

# Rendimiento
rainy-cli analyze -p src/ -a performance

# Estilo y mantenibilidad
rainy-cli analyze -p . -a style

# Complejidad
rainy-cli analyze -p src/main.rs -a complexity
```

### Análisis con Sesiones

```bash
# Crear sesión dedicada para análisis
rainy-cli session create "Análisis de Seguridad" --description "Revisión de vulnerabilidades"
rainy-cli session add-tag <session_id> "security" "audit"

# Realizar análisis
rainy-cli session chat <session_id> "analiza vulnerabilidades en auth.rs" --context-file src/auth.rs
rainy-cli session chat <session_id> "revisa las mejores prácticas de seguridad"
```

---

## 💻 Generación de Código

### Generación Básica

```bash
# Generar código desde descripción
rainy-cli generate "crea una función para validar emails en Rust"

# Con opciones adicionales
rainy-cli generate "implementa un logger asíncrono" --with-tests --with-docs
```

### Generación con Sesiones

```bash
# Crear sesión para desarrollo de feature
rainy-cli session create "Feature: User Management" --description "Sistema de gestión de usuarios"
rainy-cli session add-tag <session_id> "feature" "users"

# Generar componentes paso a paso
rainy-cli session chat <session_id> "diseña las estructuras de datos para User"
rainy-cli session chat <session_id> "implementa el repositorio de usuarios"
rainy-cli session chat <session_id> "crea los handlers HTTP para CRUD"
```

---

## 📝 Revisión de Código

### Revisión Básica

```bash
# Revisar archivo específico
rainy-cli review -p src/main.rs

# Revisar con enfoque específico
rainy-cli review -p src/ -f performance
rainy-cli review -p src/auth.rs -f security
rainy-cli review -p . -f readability
```

### Revisión Git-Aware

```bash
# Revisar cambios en Git
rainy-cli review --git --git-ref HEAD~1

# Revisar rama específica
rainy-cli review --git --git-ref origin/main

# Revisar con enfoque
rainy-cli review --git -f security
```

### Revisión con Sesiones

```bash
# Crear sesión para code review
rainy-cli session create "Code Review Sprint" --description "Revisión de código del equipo"
rainy-cli session add-tag <session_id> "review" "sprint"

# Realizar revisiones
rainy-cli session chat <session_id> "revisa estos cambios de autenticación" --context-file src/auth.rs
rainy-cli session chat <session_id> "sugiere mejoras en el manejo de errores"
```

---

## 🏗️ Gestión de Proyectos

### Plantillas de Proyecto

```bash
# Crear proyecto desde plantilla
rainy-cli template rust-api mi-api-proyecto
rainy-cli template rust-cli mi-cli-proyecto
rainy-cli template rust-lib mi-libreria
rainy-cli template web-api mi-api-web
rainy-cli template microservice mi-microservicio
```

### Desarrollo con Sesiones

```bash
# Sesión por módulo/feature
rainy-cli session create "Módulo: Autenticación" --description "Sistema de auth con JWT"
rainy-cli session create "Módulo: Base de Datos" --description "ORM y migraciones"
rainy-cli session create "Módulo: API Endpoints" --description "REST API endpoints"

# Trabajo colaborativo
rainy-cli session export <session_id> auth_module_backup.json
# Compartir con equipo y luego importar
rainy-cli session import auth_module_backup.json --name "Auth - Colaborativo"
```

---

## ⚙️ Configuración

### Ver Configuración

```bash
# Mostrar configuración actual
rainy-cli config --show
```

### Configurar API Key

```bash
# Establecer API key
rainy-cli config --set-api-key "tu-api-key-aqui"
```

### Configurar Modelo

```bash
# Cambiar modelo predeterminado
rainy-cli config --set-model "moonshotai/kimi-k2-instruct-0905"
rainy-cli config --set-model "anthropic/claude-sonnet-4"
rainy-cli config --set-model "openai/gpt-4-turbo"
```

### Reset de Configuración

```bash
# Restaurar configuración por defecto
rainy-cli config --reset
```

---

## 💡 Mejores Prácticas

### Organización por Sesiones

```bash
# Estructura recomendada
rainy-cli session create "Proyecto: E-commerce" --description "Aplicación completa de tienda online"
  ├── "Auth Module" (tag: auth, security)
  ├── "Product Catalog" (tag: products, api)
  ├── "Shopping Cart" (tag: cart, frontend)
  └── "Payment Integration" (tag: payment, stripe)
```

### Ahorro de Tokens

```bash
# Para preguntas simples
rainy-cli chat --no-history "sintaxis básica"

# Para desarrollo continuo
rainy-cli session chat <session_id> "continúa implementando"

# Para análisis específicos
rainy-cli chat --context-file archivo.rs "analiza solo esto"
```

### Workflow Recomendado

#### Para Proyectos Grandes (Sesiones Manuales)
```bash
# 1. Crear sesión al inicio del proyecto
rainy-cli session create "Mi Nuevo Proyecto"

# 2. Etiquetar apropiadamente
rainy-cli session add-tag <id> "rust" "api" "web"

# 3. Desarrollar paso a paso
rainy-cli session chat <id> "diseña la arquitectura"
rainy-cli session chat <id> "implementa las bases de datos"
rainy-cli session chat <id> "crea los endpoints"

# 4. Backup regular
rainy-cli session export <id> proyecto_backup.json

# 5. Mantener organizado
rainy-cli session search "api"  # Encontrar sesiones relacionadas
```

#### Para Consultas Espontáneas (Sesiones Automáticas) 🆕
```bash
# ✨ Sesiones automáticas para cualquier consulta
rainy-cli chat "optimiza esta función de fibonacci"
# → Se crea automáticamente: "optimiza esta función de fibonacci"

rainy-cli chat "explica el patrón de diseño observer"
# → Se crea automáticamente: "explica el patrón de diseño observer"

# Continuar la conversación en futuras sesiones
rainy-cli session chat <generated_id> "muéstrame un ejemplo práctico"
rainy-cli session chat <generated_id> "qué otros patrones son similares"
```

### Combinaciones Útiles

```bash
# Análisis + Sesión
rainy-cli session chat <id> "analiza este código" --context-file src/main.rs

# Generación + Contexto
rainy-cli generate "función de validación" --with-tests --context-file src/validators.rs

# Revisión Git + Sesión
rainy-cli session chat <id> "revisa estos cambios" --git --git-ref HEAD~3
```

---

## 🔧 Atajos y Consejos

### Comandos Rápidos

```bash
# Ver ayuda
rainy-cli --help
rainy-cli chat --help
rainy-cli session --help

# Ver versión
rainy-cli --version
```

### Manejo de Errores

```bash
# Si hay error de API key
rainy-cli config --set-api-key "tu-nueva-key"

# Si hay problemas de sesión
rainy-cli session list  # Ver sesiones disponibles
rainy-cli session show <id>  # Ver detalles
```

### Limpieza y Mantenimiento

```bash
# Limpiar historial global
# (Eliminar ~/.rainy-cli/chat_history.json)

# Backup de sesiones importantes
rainy-cli session export <id> backup_importante.json

# Limpiar sesiones antiguas
rainy-cli session search "deprecated"
rainy-cli session delete <id_deprecated>
```

---

## 🚀 Características Destacadas

### ✨ Sesiones Automáticas con IA 🆕

**La innovación más poderosa**: creación inteligente de sesiones sin intervención manual

**¿Por qué es revolucionario?**
- 🤖 **IA genera títulos** automáticamente usando Llama-3.1-8b-instant
- 🎯 **Contexto organizado** desde el primer mensaje
- 💾 **Historial preservado** automáticamente
- 🔄 **Continuidad perfecta** entre sesiones
- ⚡ **Cero configuración** para el usuario

**Ejemplo de experiencia:**
```bash
# Antes (manual y tedioso)
rainy-cli session create "componente acordeon react"
rainy-cli session chat <id> "Crea un componente..."

# Ahora ✨ (automático e inteligente)
rainy-cli chat "Crea un componente React de acordeón usando shadcn/ui"
# → ¡Se crea todo automáticamente con IA!
```

### Comparación de Enfoques

| Característica | Sesiones Manuales | Sesiones Automáticas 🆕 |
|---|---|---|
| **Configuración** | Manual (crear, nombrar) | Automática con IA |
| **Títulos** | Usuario decide | IA genera título inteligente |
| **Modelo usado** | Modelo predeterminado | Mismo modelo predeterminado |
| **Velocidad** | 2-3 comandos | 1 comando directo |
| **Organización** | Manual | Automática |
| **Contexto** | Manual | Automático |
| **Eficiencia** | Buena | Excelente |

---

## 📊 Estadísticas y Monitoreo

### Verificar Uso de Tokens

Los comandos muestran automáticamente:
```
TOK Tokens: [Prompt: 1234, Completion: 567, Total: 1801]
Model: moonshotai/kimi-k2-instruct-0905
Speed: 2.5s
```

### Comparación de Eficiencia

- **Sin sesiones**: ~3500 tokens promedio
- **Con sesiones**: ~1500-2000 tokens promedio
- **Sesiones automáticas**: ~1200-1800 tokens promedio
- **Prompts cortos**: ~800-1200 tokens
- **Ahorro**: hasta 60% menos tokens

### Modelos Utilizados

- **Conversaciones principales**: `moonshotai/kimi-k2-instruct-0905`
- **Generación de títulos y descripciones**: `moonshotai/kimi-k2-instruct-0905` (mismo modelo para consistencia)
- **Fallback automático** si hay problemas de conectividad

---

## 🎯 Resumen Ejecutivo

### ✨ Nueva Era: Sesiones Automáticas

**Rainy CLI ha evolucionado** con la funcionalidad más inteligente del mercado:

#### Para TODOS los Usuarios (Principiantes y Avanzados)
```bash
# ✨ La forma más simple y poderosa
rainy-cli chat "tu pregunta o tarea"
# → ¡IA crea automáticamente la sesión perfecta!
```

### Comparación: Antes vs Ahora

| Aspecto | Antes (Manual) | Ahora ✨ (Automático) |
|---|---|---|
| **Primer comando** | `session create` + `session chat` | `chat "mensaje"` |
| **Títulos** | Usuario piensa nombre | IA genera título inteligente |
| **Organización** | Manual con tags | Automática + IA |
| **Velocidad** | 2-3 comandos | 1 comando directo |
| **Experiencia** | Técnica | Conversacional |

### Estrategias de Uso

#### 🚀 Para Consultas Rápidas
```bash
rainy-cli chat "¿Cómo funciona async en Rust?"
rainy-cli chat "optimiza esta función SQL"
# → Sesiones automáticas creadas al instante
```

#### 🏗️ Para Desarrollo de Proyectos
```bash
# Sesiones manuales para proyectos grandes
rainy-cli session create "E-commerce API"
rainy-cli session chat <id> "diseña arquitectura"

# O sesiones automáticas para tareas específicas
rainy-cli chat "implementa sistema de pagos"
# → Se crea sesión "implementa sistema de pagos"
```

#### 🎯 Para Trabajo Colaborativo
```bash
# Exportar sesiones para compartir
rainy-cli session export <id> proyecto_compartido.json

# Importar sesiones de compañeros
rainy-cli session import proyecto_equipo.json
```

### Eficiencia Máxima
- **Sesiones automáticas** para consultas espontáneas
- **Sesiones manuales** para proyectos organizados
- **`--no-history`** para preguntas simples sin contexto
- **Context files** para análisis específicos
- **Tags** para organización perfecta
- **60% menos tokens** con sesiones inteligentes

### 💡 Pro Tip
**Empieza con sesiones automáticas** para todo. Si necesitas más organización, crea sesiones manuales adicionales. ¡La IA hace el trabajo pesado por ti! 🤖✨

---

¡Listo para dominar tu flujo de desarrollo con Rainy CLI! 🚀

Para más información, usa `rainy-cli --help` o consulta la documentación en cada comando específico.

