# Cartalith Markdown Vault Integration
## Functional & Technical Design Specification

> ## Owner clarification, 2026-08-18 — read before the spec below
>
> Verbatim: *"In the spec obsidian is mentioned a lot but it should be a
> Markdown-vault (i personally use obsidian) as a stretch goal an obsidian
> plugin is a wish. But that is largely deferred for now. Keep it on the
> roadmap at the end or an extra. Its not a critical part."*
>
> Three things this settles:
>
> 1. **The target is a generic Markdown vault, not Obsidian.** The spec below
>    already says so (§0 terminology, and §10 "the integration should not
>    require Obsidian"), but it names Obsidian often enough to read as an
>    Obsidian feature. It is not. Obsidian is one compatible vault, and the
>    owner's own — nothing may *require* it, and no Obsidian-specific
>    behaviour belongs in the core.
> 2. **An Obsidian plugin is a wish, not a deliverable.** Already §33's
>    non-goal and §34's stretch goal; now confirmed as deferred outright.
> 3. **Priority is low.** "Not a critical part" — it stays at the end of
>    `ROADMAP.md`, under "Options kept open, not scheduled", and does not
>    compete with engine or parity work.
>
> **This resolves a real conflict `DCC_CONTROL_INDEX.md` raised.** The newly
> imported `DCC_SHELL_SPEC.md` §9 puts a *MARKDOWN VAULT · LINKED* block in
> the Data manager that assumes more than this document scopes: `obsidian://`
> links written into exported tiles, note links inside exported GeoJSON, and a
> **two-way sync toggle** — which is an explicit V1 non-goal here (§33). Under
> this clarification the spec's `obsidian://` scheme is Obsidian-specific and
> therefore out of core, and two-way sync stays a non-goal. Whoever builds §9
> should treat that block as **deferred**, not as an approved requirement, and
> the vault path/note-count readout is the only part of it consistent with V1.
>
> **Refined spec and templates are coming from the owner** — expect this
> document to be replaced or extended. Do not start implementation against the
> version below.

> Owner-supplied design (2026-08-18), imported verbatim — "Here is another
> function for on the roadmap." Recorded as a future, unscheduled phase
> (`ROADMAP.md`'s own "Options kept open" section) — a genuinely new feature,
> not a port of anything in the reference HTML app, so it sits outside
> `DECISIONS.md` §7d's contract entirely (nothing to preserve, nothing to
> modernize-over — this is net-new scope). Not started; no code exists for
> this yet, and nothing here has been cross-checked against the current Rust/
> Godot architecture beyond this note. Whoever picks this up should write a
> real `MARKDOWN_VAULT_SCOPE.md` first, the same discipline every other large
> effort in this project has followed (`JOURNEY_PLANNER_SCOPE.md`,
> `ASSET_LIBRARY_SCOPE.md`, `UNIFIED_TOOL_PLAN.md`) — in particular, V1's
> acceptance criteria (§35) assume entity concepts (settlements, POIs,
> regions) and an entity-information-panel UI that this port has some of
> (settlements are real; POIs/regions as first-class addressable entities
> with their own panel may not exist yet — verify before scoping).

**Status:** Proposed design
**Scope:** V1 implementation
**Primary platforms:** Windows and Android
**Future:** Obsidian/editor integrations are explicitly deferred
**Terminology:** "Markdown Vault" is the generic term. An Obsidian vault is treated as one compatible Markdown Vault.

---

## 1. Purpose

Cartalith is a cartographic, simulation, worldbuilding, and storytelling application. The Markdown Vault integration connects Cartalith's spatial/world-state model with an external corpus of Markdown worldbuilding documents without importing that corpus into the Cartalith project.

The integration shall allow users to:

1. Connect an external Markdown Vault.
2. Browse and read Markdown files from that vault.
3. Attach whole documents or selected sections to Cartalith entities.
4. Edit imported text inside Cartalith when desired.
5. Track the relationship between an imported section and its source document.
6. Detect source changes using file timestamps/metadata.
7. Keep the Markdown Vault external to the Cartalith save by default.
8. Optionally store a user-selected copy of vault material according to the project's chosen storage policy.
9. Generate a controlled Cartalith information block for a mapped entity.
10. Generate a map screenshot around that entity.
11. Let the user explicitly select which Cartalith information is included.
12. Explicitly write generated information back to the Markdown Vault only when the user requests it.

V1 is deliberately **pull-oriented**. Cartalith reads from the Markdown Vault. Automatic bidirectional synchronization is deferred.

---

# 2. Core Design Principle

Cartalith must not become a second writing application or a replacement for the Markdown editor.

The separation is:

- **Markdown Vault:** authoring and narrative knowledge.
- **Cartalith:** spatial, geographic, systemic, and simulation state.
- **Integration:** controlled relationship between the two.

The Markdown Vault remains external unless the user deliberately chooses a project-local storage mode.

Cartalith must never require the presence of the original vault merely to open a project.

---

# 3. V1 Entity Scope

Markdown links are supported for:

- Settlements
- POIs
- Regions
- Region labels

The underlying implementation should use a generic `KnowledgeLink` / `MarkdownLink` entity association so additional Cartalith entity types can be added later without redesigning the storage model.

Future candidates include:

- Routes
- Factions
- Characters
- Events
- Timeline entries

These are outside V1.

---

# 4. Architecture

```text
                         CARTALITH
                            |
                    Markdown Vault Layer
                            |
              +-------------+-------------+
              |                           |
       Vault Provider                Markdown Parser
              |                           |
       +------+-------+           +-------+-------+
       |              |           |               |
    Windows        Android     Core Markdown   Obsidian
 filesystem          SAF       + selected       constructs
                              constructs
              |
              v
       Knowledge Links
              |
      +-------+--------+
      |       |        |
 Settlement  POI     Region
              |
              v
       Feedback / Export
              |
      +-------+--------+
      |                |
  Cartalith data    Map snapshot
              |
              v
       Explicit Markdown write
```

The architecture shall separate:

1. **Vault access**
2. **Markdown parsing**
3. **Cartalith entity linking**
4. **Local imported-text state**
5. **Feedback/export generation**
6. **Platform-specific file permissions**

No platform-specific filesystem implementation should leak into Cartalith's domain model.

---

# 5. Markdown Vault Identity

A vault is represented independently of its physical filesystem path.

Recommended conceptual identity:

```text
Vault
- vaultId
- displayName
- providerType
- platformBinding
- root metadata
```

A linked file is identified by:

```text
VaultFile
- vaultId
- relativePath
- lastKnownModifiedTimestamp
- optional content hash
```

The absolute path or Android document URI is a platform binding, not the semantic identity of the file.

Example:

```text
vaultId:
    vault_7f31...

relativePath:
    Locations/Nareth.md
```

Windows may resolve this to:

```text
D:\World\Elaris\Locations\Nareth.md
```

Android may resolve it through a Storage Access Framework directory URI.

The Cartalith project must not depend on the Windows path.

---

# 6. Platform Vault Provider

Define a platform-neutral provider interface.

Conceptually:

```text
MarkdownVaultProvider

connectVault()
disconnectVault()

listDirectory()
findFiles()
readFile()
getFileMetadata()
writeFile()
createFile()
openExternal()
```

Optional capabilities:

```text
watchChanges()
moveFile()
deleteFile()
```

These should be capability-based because Android and desktop filesystem semantics differ.

## Windows

Use the native filesystem.

The user selects or provides the Markdown Vault root.

## Android

Use Android's Storage Access Framework or the application's native equivalent.

The user explicitly grants Cartalith access to a directory.

The resulting Android permission/binding remains device-specific.

---

# 7. Vault Connection

The user flow:

```text
Markdown Vault
    |
    +-- Connect Vault
            |
            +-- Select directory
            |
            +-- Validate
            |
            +-- Assign vaultId
            |
            +-- Connected
```

On another device:

```text
Cartalith project
    |
    +-- Existing vault reference
            |
            +-- Not connected
                    |
                    +-- Connect Existing Vault
                    |
                    +-- Select directory
                    |
                    +-- Verify expected files
                    |
                    +-- Connected
```

The application must clearly distinguish:

- Connected
- Temporarily unavailable
- Missing source
- Stale source
- Never connected

---

# 8. Vault Storage Policy

The user must control where Markdown-derived material is stored.

Supported conceptual modes:

### External Vault

The Markdown Vault remains entirely outside the Cartalith project.

Cartalith stores references and optional cached data.

### Project-Local Copy

The user may deliberately choose to copy relevant Markdown material into the current Cartalith project's save structure.

This is not the default.

### User-Defined Structure

The user may choose a custom location/structure for Cartalith-generated material.

Cartalith must not silently create a `.cartalith` directory in the vault without user consent.

---

# 9. Markdown Reading

Cartalith should read Markdown on demand rather than indexing and loading the entire vault into memory.

Required V1 operations:

- Read file
- Parse Markdown
- Identify headings
- Identify sections
- Read frontmatter
- Render supported Markdown
- Preserve source text
- Detect source modification

The application should avoid keeping the complete vault in memory.

Large vaults must remain practical.

---

# 10. Markdown Compatibility

The core parser shall support standard Markdown.

V1 should additionally understand common Obsidian-compatible constructs where practical:

- Wikilinks
- Tags
- YAML frontmatter
- Callouts
- Embeds
- Markdown links
- Images
- Headings
- Lists
- Code blocks
- Block references where feasible

Unsupported constructs should be preserved as source text rather than destroyed or silently rewritten.

The integration should not require Obsidian.

---

# 11. Knowledge Links

Introduce a generic association:

```text
KnowledgeLink
- linkId
- entityId
- entityType
- vaultId
- relativePath
- selection
- sourceTimestamp
- optional sourceHash
- importedText
- editedText
- status
```

`selection` can be:

```text
WholeDocument
HeadingSection
TextRange
MarkdownBlock
```

V1 UI should prioritize:

1. Whole document
2. Heading/section
3. Arbitrary selected text

The implementation must preserve enough source information to identify the original section later.

---

# 12. Whole Document Import

The user may import the full Markdown file into Cartalith.

The imported content becomes a local working copy associated with the source document.

Example:

```text
Nareth.md
    |
    +-- imported into Settlement: Nareth
```

The user may edit that imported text inside Cartalith.

Those edits are **not automatically written back** to the Markdown Vault.

---

# 13. Section Import

The user may select:

```text
# Nareth

## History

...

## The Old Quarter

...
```

and attach only:

```text
## The Old Quarter
```

The link records the source section and its source timestamp.

If the user edits the imported section inside Cartalith, those edits belong to the Cartalith-side working copy until explicitly exported.

---

# 14. Source Synchronization

V1 synchronization is one-way:

```text
Markdown Vault
       |
       v
   Cartalith
```

No automatic write-back.

The primary source-change detection mechanism is file modification timestamp.

Recommended additional protection:

```text
modifiedTimestamp
+
optional content hash
```

When a linked source is opened or refreshed:

```text
Stored timestamp
       vs.
Current timestamp
```

If changed:

```text
Source changed
----------------------------
Nareth.md has changed since
it was imported.

[Reload source]
[Keep current copy]
[Compare]
```

The exact conflict UI can be expanded later.

---

# 15. Edited Imported Text

Imported text has two conceptual states:

```text
Source text
Cartalith working text
```

If the user modifies the imported text:

```text
source:
    Nareth.md / ## History

working copy:
    edited version
```

Cartalith records that the working copy diverges from the source.

The user can then explicitly choose:

```text
Insert Updated Text into Source
```

This action is the only V1 path that writes the edited Markdown section back to the vault.

---

# 16. Partial Update Back to Source

The update operation must be section-aware.

It should not overwrite the entire Markdown document when only a section has changed.

Conceptually:

```text
Source file
    |
    +-- unchanged content
    |
    +-- target section
    |       replaced with
    |       Cartalith working version
    |
    +-- unchanged content
```

Before writing:

1. Re-check source timestamp.
2. Confirm the target section still exists.
3. Verify the source has not changed unexpectedly.
4. Show a preview.
5. Require explicit confirmation.
6. Write only the selected section.
7. Refresh stored source metadata.

If the source changed in the meantime, Cartalith must not blindly overwrite it.

---

# 17. Explicit Write Principle

All writes to the Markdown Vault are explicit user actions.

Examples:

```text
[Update Source Section]
[Write Cartalith Information]
[Save Map Snapshot]
```

There is no silent background writing in V1.

Reading can be automatic/on-demand.

Writing cannot.

---

# 18. Cartalith Feedback System

The feedback system generates a controlled block of Cartalith-derived information for a linked entity.

Example:

```markdown
<!-- CARTALITH:BEGIN entity="settlement_0042" version="1" -->

## Cartalith

![Nareth map](...)

**Settlement**
- Type: River Town
- Population: 8,420

**Geography**
- Region: Lower Nareth
- Elevation: 34 m
- Biome: Temperate River Valley

<!-- CARTALITH:END -->
```

The block is machine-owned.

Everything outside it belongs to the user.

---

# 19. Export Data Model

Exportable information shall not be hardcoded into the Markdown UI.

Cartalith should expose a registry of exportable data fields.

Suggested groups:

## Identity

- Name
- Entity type

## Geography

- Region
- Coordinates
- Elevation
- Biome
- Climate
- Terrain

## Settlement

- Settlement type
- Population
- Government
- Economy
- Culture
- Faction

## Infrastructure

- Roads
- Rivers
- Trade connections
- Nearby settlements

## Map

- Immediate map
- Local map
- Regional map

## Navigation

- Cartalith entity reference
- Coordinates
- Open-in-Cartalith link

Only fields actually available for the selected entity should be offered.

---

# 20. Export Selection UI

Example:

```text
UPDATE MARKDOWN
────────────────────────

Entity: Nareth

Identity
☑ Name
☑ Entity type

Geography
☑ Region
☑ Coordinates
☑ Elevation
☑ Biome
☐ Climate
☐ Terrain

Settlement
☑ Settlement type
☑ Population
☐ Government
☐ Economy
☐ Culture
☐ Faction

Infrastructure
☐ Roads
☐ Rivers
☐ Trade connections
☐ Nearby settlements

Map
☑ Immediate map
☐ Local map
☐ Regional map

[Preview]
[Cancel] [Write to Markdown]
```

The UI must not expose information that the entity does not possess.

---

# 21. Map Snapshot

V1 shall reuse Cartalith's current renderer.

The generated map is effectively a screenshot/crop of the current Cartalith map.

There is no separate export renderer in V1.

The user can choose:

- Immediate
- Local
- Regional

The exact radius/scale may be configurable.

The generated image should represent the current active map state.

Future versions may add:

- layer selection
- dedicated export styling
- cartographic themes
- print-oriented output
- custom annotations

These are outside V1.

---

# 22. Generated Image Storage

Two storage strategies are supported conceptually:

```text
User-selected location
```

or:

```text
Project-local generated assets
```

If stored in a Markdown Vault, Cartalith should propose a predictable structure such as:

```text
.cartalith/
    maps/
        settlement_0042.png
```

but the user must explicitly accept the proposed structure or choose another location.

The integration must not silently pollute the Markdown Vault.

Base64-embedding large images directly into Markdown is not recommended for V1.

---

# 23. Machine-Owned Markdown Block

Cartalith-generated information must be delimited.

Recommended:

```markdown
<!-- CARTALITH:BEGIN entity="settlement_0042" version="1" -->

...

<!-- CARTALITH:END -->
```

Rules:

1. Cartalith owns only the delimited block.
2. User content outside the block is immutable from Cartalith's perspective.
3. Updates replace only the Cartalith block.
4. If the block cannot be safely identified, Cartalith must not overwrite arbitrary content.
5. The user receives a preview before writing.

This provides deterministic regeneration.

---

# 24. Existing Cartalith Block Handling

When updating an entity:

```text
Block exists
    |
    +-- Replace existing block
```

If it does not:

```text
No block
    |
    +-- Preview insertion location
    |
    +-- User confirmation
    |
    +-- Insert block
```

Insertion location should eventually be configurable.

V1 may use a predictable location, such as after the document's main title/frontmatter.

---

# 25. Separate JSON Configuration/Data Layer

Cartalith's exportable-information definitions should be data-driven.

The base definitions should live in their own JSON structure rather than being embedded directly in Markdown.

Conceptually:

```text
Cartalith
│
├── World/Map data
│
├── Markdown Vault integration
│
│   ├── vault references
│   ├── knowledge links
│   └── imported working copies
│
└── Export definitions
    └── cartalith-markdown-fields.json
```

The Markdown Vault is read separately.

The two are presented together in the UI, but they remain distinct data sources.

This separation is important for maintainability and future extensibility.

---

# 26. Save-File Model

A Cartalith project should contain references similar to:

```json
{
  "markdownVault": {
    "vaultId": "vault_7f31",
    "displayName": "Elaris",
    "binding": null
  },
  "knowledgeLinks": [
    {
      "entityId": "settlement_0042",
      "entityType": "settlement",
      "vaultId": "vault_7f31",
      "relativePath": "Locations/Nareth.md",
      "selection": {
        "type": "heading",
        "value": "The Old Quarter"
      },
      "sourceTimestamp": 1786942214
    }
  ]
}
```

The platform-specific vault binding should not be treated as portable world data.

---

# 27. Offline / Missing Vault Behaviour

Cartalith must remain usable if the vault is unavailable.

States:

### Connected

Source available.

### Cached

Source unavailable, but imported text is available.

### Stale

Source timestamp differs from the last known state.

### Missing

Expected source cannot be found.

### Unbound

The project knows about the vault but this device has not connected it.

The map and simulation remain functional in all cases.

---

# 28. UI Integration

The Markdown functionality should appear in entity information panels rather than as an isolated utility.

For a settlement:

```text
NARETH
────────────────────

Settlement
Population: 8,420
Region: Lower Nareth

KNOWLEDGE
────────────────────

Nareth.md
  ✓ Connected

  Attached:
  ├─ Whole document
  └─ The Old Quarter

[Open Markdown]
[Attach Markdown]
[Refresh]

CARTALITH FEEDBACK
────────────────────

[Configure / Preview]
[Write to Markdown]
```

The same component can be used by POIs and regions.

---

# 29. Reader / Working Copy UI

When imported text is selected:

```text
NARETH — THE OLD QUARTER

Source:
Locations/Nareth.md
Section:
## The Old Quarter

Source modified:
16 Aug 2026 21:14

────────────────────────

[Markdown content]

────────────────────────

Status:
✓ Matches source

[Edit]
[Reload]
```

After editing:

```text
Status:
● Local changes

[Save Local Copy]
[Compare With Source]
[Insert Updated Section]
```

The source remains untouched until the explicit update action.

---

# 30. Security and Permission Rules

The integration must follow least privilege.

Cartalith should request access only to the selected Markdown Vault.

It must not recursively access unrelated user directories.

On Android, access should be granted through the platform's document/directory permission mechanism.

Writing requires:

1. Existing permission
2. Valid source
3. Explicit user command
4. Confirmation where appropriate

---

# 31. Performance Requirements

The integration must not become a performance burden on Cartalith.

Requirements:

- Do not load the entire vault into memory.
- Do not parse every Markdown file at startup.
- Use lazy loading.
- Cache only attached/imported content.
- Use timestamps to avoid unnecessary rereads.
- Hash content only when necessary.
- Keep generated map images outside the main world-data structures.
- Avoid blocking the main UI thread on large file operations.
- Use platform-native asynchronous filesystem operations where possible.

This is particularly important for Android.

---

# 32. Failure Handling

The system must explicitly handle:

- file deleted
- file moved
- vault moved
- permission revoked
- file modified externally
- section deleted
- heading renamed
- duplicate headings
- malformed Markdown
- unsupported Markdown construct
- failed write
- insufficient permissions
- generated image unavailable
- stale Cartalith block

No destructive fallback should occur.

When uncertain, Cartalith should stop and ask the user rather than guessing.

---

# 33. V1 Non-Goals

Do not implement in V1:

- automatic bidirectional synchronization
- automatic Markdown writes
- Obsidian plugin
- editor extensions
- vault-wide semantic indexing
- automatic graph construction
- cloud vault synchronization
- Git integration
- arbitrary Markdown rewriting
- dedicated map export renderer
- advanced export styling
- automatic conflict merging

These should remain future extensions.

---

# 34. Future Integration Layer

The architecture should leave room for:

```text
Cartalith Markdown Protocol
        |
        +-- Obsidian adapter
        +-- VS Code adapter
        +-- Other Markdown editors
```

The Obsidian integration is a stretch goal.

It should consume the same underlying Markdown Vault/Cartalith relationship rather than becoming a special case inside the core engine.

---

# 35. Acceptance Criteria

V1 is functionally complete when a user can:

1. Connect a Markdown Vault on Windows.
2. Connect the same logical vault on Android using a different platform path/binding.
3. Browse Markdown files.
4. Open a Markdown file.
5. Attach a complete file to a settlement.
6. Attach a specific section to a POI.
7. Attach a region document to a region.
8. Import text into Cartalith.
9. Edit the imported text locally.
10. Detect that the source file has changed using its timestamp.
11. Compare or reload changed source content.
12. Explicitly insert an edited section back into the source Markdown.
13. Select Cartalith information groups for export.
14. Generate an immediate/local/regional screenshot using the existing renderer.
15. Preview the generated Markdown block.
16. Explicitly write that block to the Markdown Vault.
17. Update an existing Cartalith block without altering surrounding user content.
18. Open the Cartalith project when the vault is unavailable.
19. Show the user which information is cached, stale, missing, or connected.
20. Keep Markdown-derived data and Cartalith's own world data logically separated.

---

# 36. Engineering Principle

The implementation should preserve this relationship:

```text
                 WORLD
                   |
          +--------+--------+
          |                 |
      Cartalith         Markdown Vault
          |                 |
   spatial/systemic      narrative
      truth              knowledge
          |                 |
          +-------+---------+
                  |
             Knowledge Link
```

Neither side should silently become the other.

Cartalith provides **where, what, how much, how connected, and how the world behaves**.

The Markdown Vault provides **what people know, what happened, what places mean, and how the world is described**.

V1 connects those two representations through explicit, inspectable, reversible actions.
