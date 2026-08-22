# Changelog

## 1.0.0 - 2026-08-22

- Añade traducciones completas en portugués, alemán y francés, además de inglés y español.
- Añade portal por servidor con permisos propios y aislamiento estricto por usuario y servidor.
- Añade gestión completa de departamentos, respuestas guardadas, asignación a personal y etiquetas.
- Añade filtros administrativos por responsable, servidor, usuario, SLA, fechas y etiquetas.
- Añade solicitudes de acceso temporal con consentimiento, sincronización con Wings, revocación, caducidad y restauración de permisos previos.
- Migra los adjuntos nuevos al almacenamiento privado nativo, compatible con filesystem y S3, y conserva lectura retrocompatible de adjuntos antiguos.
- Envía notificaciones reales mediante las plantillas y el servicio de correo nativos del panel.
- Añade historial auditable de estados, asignación, etiquetas y ciclo de acceso.
- Conserva la referencia legible del servidor cuando este se elimina y revoca sus accesos pendientes o activos.

- Cambia la identidad completa de desarrollo a `team.voidvalue.tickets`.
- Traduce todos los textos propios de la pantalla de configuración al inglés base y al español.
- Impide que un cliente cambie el estado de tickets ajenos mediante un UUID conocido.
- Limita los cambios de estado del cliente a cerrar y reabrir.
- Valida en backend los límites de configuración y la coherencia de la duración de acceso.
- Amplía las pruebas unitarias de estados, payloads, paginación y configuración.

- Convierte la experiencia en una única cola de soporte global.
- Añade selección opcional del servidor afectado al crear un ticket.
- Hace que las filas sean pulsables y añade vista completa de conversación.
- Permite responder, crear notas internas y actualizar, cerrar o reabrir estados según el scope.
- Muestra cliente, servidor y autores mediante nombres legibles sin migrar los tickets existentes.
- Retira la sección duplicada de soporte dentro de cada servidor.
- Corrige la selección de servidores para administradores sin servidores propios.
- Añade categorías de tickets abiertos, cerrados y todos, además de estadísticas de cola.
- Añade adjuntos privados con límites de tipo, tamaño y cantidad.
- Añade borrado permanente de tickets para administradores con permiso y confirmación explícita.

- Cambia la compatibilidad mínima a Calagopus Panel 1.1.4.
- Valida backend y frontend contra el commit oficial de 1.1.4.
- Retira Quick Actions, cuyo registry todavía no existe en Calagopus 1.1.4.
- Mantiene las rutas nativas de cuenta, servidor y administración.

- Primera versión exportable del sistema de soporte y tickets.
