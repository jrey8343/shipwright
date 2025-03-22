#Shipwright
Craft. Launch. Scale.
An opinionated full-stack Rust meta-framework designed to help you ship production-ready apps — fast.

Shipwright helps you scaffold, build, and maintain modern web applications using a best-of-Rust toolkit: Axum, SeaORM, HTMX, Minijinja, and more — all wrapped in a powerful CLI with zero boilerplate and batteries included.

##Features at a glance:
Database-ready via SeaORM – async ORM with full Postgres, MySQL, and SQLite support

- Automatic model & changeset validation – inspired by Loco’s Validatable and Gerust’s faker

- Telemetry via Tracing – built-in instrumentation from day one

- Fast and clean HTTP architecture with Axum – including WebSockets and background Channels

- Split APIs – structured JSON and app-layer APIs for HTMX-style interactions (read why)

- Real-time jobs & tasks via Apalis – optionally with dashboard (apalis-board) or CLI-based job admin

- Authentication with Tower Sessions – secure, session-based auth made easy

- Flexible mailer/notifier system – SMTP-compatible, vendor-agnostic

- Minijinja templating + SSR with Enhance WASM – ergonomic templates and web components that compile to native

- Live reload & DX tooling – hot reload via minijinja_autoreload, tower_livereload, and change notifiers

- Asset bundling – support for Manganis (or other tools) to optimize, inline, and cache-bust

- Built-in Tailwind CSS support – via standalone CLI, seamlessly integrated

- AlpineJS + HTMX – simple, declarative interactivity with minimal client-side JS

- App generation & scaffolding – via cargo generate and Shipwright's custom blueprint system

- Built-in testing support – with Axum test helpers and SeaORM test tools

- Project-level documentation – auto-generated and accessible via cargo shipwright doc
