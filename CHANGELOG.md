# Changelog

## 1.1.0 - 2026-08-22

- Convierte la experiencia en una única cola de soporte global.
- Añade selección opcional del servidor afectado al crear un ticket.
- Hace que las filas sean pulsables y añade vista completa de conversación.
- Permite responder, crear notas internas y actualizar, cerrar o reabrir estados según el scope.
- Muestra cliente, servidor y autores mediante nombres legibles sin migrar los tickets existentes.
- Retira la sección duplicada de soporte dentro de cada servidor.
- Corrige la selección de servidores para administradores sin servidores propios.
- Añade categorías de tickets abiertos, cerrados y todos, además de estadísticas de cola.
- Añade adjuntos privados con límites de tipo, tamaño y cantidad.
- Corrige la cabecera multipart para que el navegador genere el boundary de cada subida.
- Añade selector de agentes y valida que solo se asignen miembros elegibles del equipo de soporte.
- Añade borrado permanente de tickets para administradores con permiso y confirmación explícita.

## 1.0.1 - 2026-08-22

- Cambia la compatibilidad mínima a Calagopus Panel 1.1.4.
- Valida backend y frontend contra el commit oficial de 1.1.4.
- Retira Quick Actions, cuyo registry todavía no existe en Calagopus 1.1.4.
- Mantiene las rutas nativas de cuenta, servidor y administración.

## 1.0.0 - 2026-08-22

- Primera versión exportable del sistema de soporte y tickets.
