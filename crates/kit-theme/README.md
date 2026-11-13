# `kit-theme`

Einheitliches Slint-Theme-System (`Hell` / `Dunkel` / `System`) für Desktop-Anwendungen auf Basis von `slint-desktop-kit`.

## Grundsätze

1. **Ein globaler Zustand:** `export global AppTheme` in Slint. Ein Aufruf `AppTheme.apply(mode)` hält `Palette.color-scheme` synchron, sodass sowohl `std-widgets` als auch eigene Komponenten konsistent umschalten.
2. **Keine Hex-Literale in App-UI:** UI-Chrome (Hintergründe, Rahmen, Texte, Akzente, Statusfarben) nutzt ausschließlich `AppTheme.*`-Tokens. Erlaubte Ausnahmen sind nur `kit-theme` selbst, Farb-Preset-Paletten (z. B. `ColorComboBox`) und nutzerdefinierte Inhaltsfarben.
3. **Paarweise verwenden:** Flächen mit `AppTheme.bg-*` immer mit `AppTheme.text-*` kombinieren.
4. **Inhaltsfarben ≠ Theme-Farben:** Nutzerdefinierte Zell-/Highlight-Farben bleiben außerhalb von `AppTheme`; für deren Textfarbe `kit_slint::color::contrast_text()` verwenden.

## Tokens (`AppTheme`)

| Token | Rolle | Dark | Light |
| :--- | :--- | :--- | :--- |
| `bg-canvas` | Fenster-Hintergrund | `#0f172a` | `#f8fafc` |
| `bg-surface` | Karten, Panels, Tabellen | `#1e293b` | `#ffffff` |
| `bg-elevated` | Dialoge, Popups, Dropdowns | `#273449` | `#ffffff` |
| `bg-subtle` | Header-Zeilen, Zebra, Eingabeflächen | `#172033` | `#f1f5f9` |
| `bg-hover` | Hover-Zustand | `#334155` | `#e2e8f0` |
| `bg-selected` | Auswahl (Zeile, Tab) | `#1e3a5f` | `#dbeafe` |
| `bg-scrim` | Modal-Overlay | `#000000aa` | `#0f172a66` |
| `border` | Standardrahmen, Trenner | `#334155` | `#e2e8f0` |
| `border-strong` | Fokus-/betonte Rahmen | `#475569` | `#cbd5e1` |
| `text-primary` | Haupttext | `#e2e8f0` | `#0f172a` |
| `text-secondary` | Labels, Nebeninfo | `#94a3b8` | `#475569` |
| `text-muted` | Platzhalter, deaktiviert | `#64748b` | `#94a3b8` |
| `accent` | Links, Primär-Akzent, aktive Elemente | `#38bdf8` | `#1d4ed8` |
| `accent-hover` | Akzent Hover | `#7dd3fc` | `#1e40af` |
| `accent-bg` | Dezente Akzentfläche (Pills, Info-Boxen) | `#38bdf81f` | `#2a7fd412` |
| `accent-fg` | Text auf vollflächigem Akzent | `#0f172a` | `#ffffff` |
| `success` / `success-bg` | OK / Erfolg | `#34d399` / `#05966926` | `#059669` / `#d1fae5` |
| `warning` / `warning-bg` | Warnung | `#fcd34d` / `#451a03` | `#b45309` / `#fef3c7` |
| `danger` / `danger-bg` | Fehler | `#f87171` / `#7f1d1d55` | `#dc2626` / `#fee2e2` |
| `info` / `info-bg` | Hinweis | `#60a5fa` / `#1e3a8a55` | `#2563eb` / `#dbeafe` |

Layout-Konstanten: `radius-sm` (`4px`), `radius-md` (`6px`), `radius-lg` (`10px`), `spacing-xs..lg` (`4/8/12/16px`), `font-size-sm..xl` (`11/13/15/18px`).

## Komponenten

- `KitCard`: `Rectangle` mit `bg-surface`, `border`, `radius-md`.
- `KitBadge`: Status-Pill (`kind`: 0 = neutral, 1 = success, 2 = warning, 3 = danger, 4 = info).
- `KitThemeSelector`: Drei Buttons (`System`, `Hell`, `Dunkel`), ruft `AppTheme.apply()` und `callback changed(string)` auf.
