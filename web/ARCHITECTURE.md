# Architecture Overview

This document outlines the `web/` structure, design patterns, and organization guidelines for this SvelteKit application.

---

## Component Tiering & Directory Structure

### 1. Base Components (`src/lib/components/`)
Focus strictly on layout, atomic UI, and visual presentation (e.g., `Button.svelte`, `Input.svelte`). 
* **Rule:** Must remain decoupled from domain-specific business logic or stores to maximize reuse.
* **Rule:** Cannot import from `widgets/` or route-level files.

### 2. Widgets (`src/lib/widgets/`)
Focus on feature-specific functionality and context. They group base components and domain logic into self-contained, functional modules (e.g., `HeaderNavigation.svelte`, `UploadFolderForm.svelte`).
* **Rule:** Can consume application state/stores and handle feature-level events.

### 3. Route Handlers (`src/routes/`)
Act as layout orchestrators and data fetchers using SvelteKit's native routing (`+page.svelte`, `+page.server.ts`).
* **Rule:** Pages should stay thin, delegating complex UI composition to Widgets and Base Components.

---

## Dependency & Import Rules

To prevent circular dependencies and maintain clean boundaries:


```
[ Routes ] ──► [ Widgets ] ──► [ Base Components ]
│                │
└────────────────┴───────► [ Stores / Utilities ]
```
