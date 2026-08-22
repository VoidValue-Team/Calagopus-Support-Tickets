# Calagopus Support / Tickets

Extensión nativa de helpdesk para Calagopus Panel. Añade una cola global de soporte con tickets de cuenta y un servidor afectado opcional, conversación con notas internas, departamentos, asignación, estados, prioridades, SLA, respuestas guardadas, auditoría, configuración, tareas automáticas y accesos desde las áreas de cuenta y administración.

El paquete se identifica como `team.voidvalue.tickets` y requiere **Calagopus Panel 1.1.4 o posterior**. Está diseñado para imágenes `heavy`, `nightly-heavy` o la variante AIO heavy equivalente. Incluye traducciones completas en inglés, español, portugués, alemán y francés.

## Estado de compatibilidad

El código se valida contra el commit de Calagopus `adc41d62ad40b0dc8afd5ffc089d1aa441dc0563` (versión 1.1.4). La integración usa exclusivamente los registries, extractors, permisos, settings, plantillas de correo, tareas y rutas publicados por Calagopus.

El acceso temporal a servidores usa el modelo nativo `ServerSubuser`, sincroniza Wings y restaura los permisos anteriores al revocar o caducar la concesión. Permanece desactivado por defecto y requiere habilitación explícita del administrador.

Los adjuntos se guardan bajo el prefijo privado de la extensión mediante el almacenamiento configurado por el panel (filesystem o S3). Solo se sirven después de volver a comprobar el acceso al ticket y nunca generan URLs públicas. Los adjuntos creados por versiones anteriores desde PostgreSQL siguen siendo legibles para facilitar la actualización.

## Arquitectura

- `backend-extensions/team_voidvalue_tickets`: extensión Rust, rutas OpenAPI, permisos, repositorio transaccional, plantillas y tareas cron.
- `frontend/extensions/team_voidvalue_tickets`: extensión React/TypeScript, rutas de cuenta, servidor y administración, configuración y traducciones.
- `database/extension-migrations/team_voidvalue_tickets`: migración PostgreSQL reversible y datos iniciales.

Las consultas de cliente se acotan siempre por `user_uuid` y las de administración exigen permisos administrativos. El servidor es una relación opcional seleccionable al crear el ticket y se valida contra los servidores accesibles por el usuario. El código de ticket se asigna mediante una secuencia PostgreSQL, evitando colisiones entre instancias. Las operaciones de creación, respuesta, estado y asignación usan transacciones.

## Instalación en Docker

1. Descarga `dist/team_voidvalue_tickets.c7s.zip` desde este repositorio.
2. Comprueba que el panel es 1.1.4+ y usa una imagen heavy.
3. En Administration → Extensions, carga el paquete y aplica la build desde la interfaz.
4. Revisa los permisos nuevos antes de conceder acceso a usuarios o subusuarios.
5. Configura el correo con los mecanismos nativos del panel; esta extensión no gestiona credenciales propias.

La instalación/rebuild puede recrear el contenedor del panel. Programa esa operación según tu política de producción.

## Desarrollo

Necesitas Rust 1.97, pnpm y un checkout de `calagopus/panel` 1.1.4+.

```bash
rsync -a backend-extensions/team_voidvalue_tickets/ \
  ../panel/backend-extensions/team_voidvalue_tickets/
rsync -a frontend/extensions/team_voidvalue_tickets/ \
  ../panel/frontend/extensions/team_voidvalue_tickets/
rsync -a database/extension-migrations/team_voidvalue_tickets/ \
  ../panel/database/extension-migrations/team_voidvalue_tickets/

cd ../panel/frontend
pnpm install
pnpm build:ci

cd ..
cargo fmt --check -p team_voidvalue_tickets
cargo clippy -p team_voidvalue_tickets --all-targets -- -D warnings
cargo test -p team_voidvalue_tickets
```

Para añadir el paquete a un entorno de desarrollo:

```bash
panel-rs extensions add path/to/team_voidvalue_tickets.c7s.zip
panel-rs extensions apply --profile balanced
```

## Exportación

Desde la raíz del checkout de Calagopus:

```bash
panel-rs extensions export team_voidvalue_tickets
```

El resultado es `exported-extensions/team_voidvalue_tickets.c7s.zip`.

## Configuración

La tarjeta de configuración permite controlar el prefijo, reapertura, prioridades del usuario, correos, límites y MIME de adjuntos, SLA/autocierre y acceso temporal. Los valores se guardan mediante `SettingsSerializeExt`; no se crean archivos de secretos.

Los departamentos incluyen prioridad por defecto, SLA de primera respuesta y resolución, autorespuesta, orden y estado. La migración crea inicialmente `Technical Support`, `Billing`, `Sales`, `Abuse` y `Other`.

## Permisos

Grupos registrados:

- Usuario: `tickets.create`, `tickets.read`, `tickets.reply`, `tickets.close`, `tickets.reopen`, `tickets.attachments`.
- Administración: lectura, respuesta, notas internas, adjuntos, borrado permanente, asignación, cambio de estado/prioridad, etiquetas, departamentos, respuestas guardadas, acceso temporal, estadísticas y settings.

La visibilidad de navegación está condicionada por permisos; no sustituye las comprobaciones del backend. Quick Actions no se registra en Calagopus 1.1.4 porque esa versión todavía no publica dicho registry.

## Email y automatización

Incluye plantillas para creación, respuestas, asignación, resolución/cierre, reapertura, alertas SLA y ciclo de acceso. Se registran con el sistema de plantillas nativo para heredar SMTP, branding y cola del panel.

Tareas multi-instancia seguras:

- barrido SLA cada minuto;
- marcado de caducidad de accesos cada minuto;
- autocierre cada hora.

Las actualizaciones masivas usan condiciones de estado/fecha en SQL y `SKIP LOCKED` para evitar dobles transiciones.

## Seguridad y privacidad

- Validación de payloads con `garde` y límites explícitos.
- SQL parametrizado; los fragmentos dinámicos solo seleccionan cláusulas fijas auditadas.
- Comprobaciones de ownership para impedir IDOR.
- Notas internas excluidas de respuestas de cliente y servidor.
- Activity log nativo para cambios sensibles.
- Correo, autenticación, ACL, almacenamiento y sincronización con Wings mediante servicios nativos del panel.
- Sin secretos ni contenido de adjuntos en logs.

El contenido de mensajes y los metadatos de adjuntos se conservan en PostgreSQL; los archivos viven en el almacenamiento privado configurado. Define una política de retención acorde a tu organización antes de habilitar el servicio.

## Actualización y desinstalación

Antes de actualizar, crea un backup de PostgreSQL y carga el nuevo `.c7s.zip` desde Extension Manager. Revisa siempre las notas de versión y migraciones.

Desinstalar una extensión no ejecuta automáticamente `down.sql` ni elimina sus datos. Esto es intencional. El rollback destructivo existe para mantenimiento controlado, pero debe ejecutarse manualmente solo después de un backup verificado.

## Capturas

Las capturas se añadirán tras validar la extensión en una instancia de Calagopus 1.1.4 con datos de demostración; no se incluyen imágenes simuladas.

## Licencia

MIT. Copyright VoidValue Team.
