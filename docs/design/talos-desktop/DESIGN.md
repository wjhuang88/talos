# Talos Desktop Visual Design Baseline

> Status: design reference for `DESKTOP-001`; no implementation authorization
>
> Scope: Talos Desktop, beginning with the Mission execution experience
>
> Related product baseline: `docs/proposals/talos-desktop-goal-oriented-workspace.md`
>
> Internationalization baseline: [`I18N.md`](I18N.md)
>
> Reference image: [`reference-execution-light.webp`](reference-execution-light.webp)

## 1. Purpose

This document defines the visual direction for Talos Desktop before GPUI implementation begins. It is intentionally narrower than a complete component library. The immediate goal is to establish a coherent visual language for the core execution experience so later Mission shaping, Evaluation, Artifact review, and Delivery surfaces inherit the same principles instead of evolving independently.

Talos Desktop must not look like a graphical TUI, a dense IDE dashboard, or a generic AI chat product. It should feel like a calm daily work tool for delegating complex work and understanding its state.

The target qualities are:

- **focused** — one obvious visual center per page;
- **calm** — low visual noise and generous whitespace;
- **connected** — layout and alignment explain how information relates;
- **trustworthy** — state, evidence, and change facts are visually distinct from narration;
- **native** — restrained material, typography, motion, and interaction rather than web-dashboard decoration;
- **Talos-consistent** — Desktop remains recognizably related to the TUI through the Nord semantic palette.

A concise visual statement is:

> A light, quiet, Nord-derived native workspace with restrained glass material and a single state-driven visual narrative.

## 2. Primary Direction

### 2.1 Light-first daily workspace

Desktop should be designed **light-first**. Dark mode remains desirable later, but the reference experience is a light daily workspace rather than a dark technical console.

Light does not mean pure white everywhere. The base should use the Nord Snow Storm family and very subtle derived tints to preserve softness and separation without introducing beige, yellow, or warm SaaS coloration.

Recommended base direction:

| Role | Direction | Canonical reference |
|---|---|---|
| App canvas | cool near-white | derived from Nord6 `#ECEFF4` |
| Primary surface | slightly brighter / more opaque | near `#F5F7FA` to white |
| Secondary material | translucent Snow Storm | Nord4–6 derived |
| Divider | low-contrast cool gray | Nord4 / derived alpha |
| Primary text | Polar Night | Nord0 `#2E3440` |
| Secondary text | Polar Night muted | Nord3 `#4C566A` |

Avoid warm cream backgrounds and orange/brown brand accents. The Desktop theme should remain visibly connected to the current TUI Nord identity.

### 2.2 Apple-like qualities, not Apple imitation

The intended reference is the restraint and hierarchy associated with high-quality native macOS applications, not a literal copy of Apple UI.

Use:

- generous whitespace;
- strong typographic hierarchy;
- large but controlled corner radii;
- subtle depth instead of heavy borders;
- translucent material where it explains hierarchy;
- simple iconography;
- calm motion tied to state transitions;
- controls that appear only when they are useful.

Do not use:

- mesh gradients;
- decorative color washes;
- glowing AI effects;
- excessive blur everywhere;
- nested glass cards;
- glossy skeuomorphic highlights;
- large dashboard metric grids.

## 3. Nord Continuity

Talos TUI currently uses Nord as its default built-in theme. Desktop should preserve the same **semantic color language**, even though the light surface model differs from the terminal implementation.

The current TUI palette defines:

```text
Polar Night
Nord0  #2E3440
Nord1  #3B4252
Nord2  #434C5E
Nord3  #4C566A

Snow Storm
Nord4  #D8DEE9
Nord5  #E5E9F0
Nord6  #ECEFF4

Frost
Nord7  #8FBCBB
Nord8  #88C0D0
Nord9  #81A1C1
Nord10 #5E81AC

Aurora
Nord11 #BF616A
Nord12 #D08770
Nord13 #EBCB8B
Nord14 #A3BE8C
Nord15 #B48EAD
```

Desktop must not blindly use the dark-theme role mapping. Instead, preserve the semantic family while selecting contrast-appropriate values for light surfaces.

Recommended light semantic mapping:

| Semantic role | Desktop direction |
|---|---|
| Primary text | Nord0 |
| Secondary text | Nord3 |
| Muted metadata | Nord3 at reduced opacity |
| Primary/current action | Nord10 |
| Secondary information | Nord9 |
| Soft informational fill | Nord8 / Nord9 at low alpha |
| Success/completed | Nord14; darkened for small text if needed |
| Failure/destructive | Nord11 |
| Warning/blocked | Nord13, with dark foreground |
| Special/evaluation | Nord15 |
| Neutral borders | Nord4 / Nord5 |

For small text and icons on light backgrounds, prefer Nord10 over Nord8 when stronger contrast is required. Nord8 should often appear as soft material, selection tint, or secondary accent rather than small primary text.

Color remains **semantic**, not decorative. Most of the page should stay neutral. Status colors should occupy small visual anchors: node markers, icon strokes, concise labels, focus state, and selected controls.

## 4. Material And Surface Model

The design may use large-radius translucent glass/material surfaces, but glass is a hierarchy tool, not the default container for every section.

### 4.1 Material hierarchy

Use at most three practical surface levels in the normal execution view:

1. **Canvas** — the continuous page background; mostly opaque and quiet.
2. **Material surface** — navigation rail, floating controls, drawers, compact change shelf, or a focused contextual region.
3. **Overlay material** — popover, command palette, modal, inspector, approval, or temporary detail surface.

Do not build:

```text
window
  -> glass page card
      -> glass section card
          -> glass row card
```

That nesting fragments the page and destroys the relationship between content.

### 4.2 Transparency

Transparency should be subtle and depend on a stable background. The visual target is a lightly frosted native material, not visible gradient texture.

Implementation guidance for GPUI experimentation:

- prefer a cool translucent neutral fill;
- pair translucency with a very subtle one-pixel edge or inner highlight;
- use shadow sparingly to distinguish floating surfaces;
- do not require blur for every panel if platform/performance constraints make it expensive;
- maintain readable fallback surfaces when transparency or blur is unavailable;
- never let transparency reduce text contrast below accessibility targets.

### 4.3 Corner radius

Large radii are part of the preferred visual character, but the radius should correlate with hierarchy.

Suggested starting tokens:

```text
window / major floating material    22–28 px
large inspector / drawer            18–22 px
compact shelf / grouped control     14–18 px
button / input                       10–14 px
small status chip                     8–10 px
```

Avoid turning every row into a pill. Rounded geometry should make the application feel soft and native without erasing structure.

## 5. Information Density And Visual Focus

A Talos page must not try to display every available state at once. The core rule is:

> One page, one dominant question.

For Execution the question is:

> **What is Talos doing now, and how is that work advancing the Mission?**

Information priority:

1. current Goal and current Work Unit;
2. position in the Mission plan;
3. recent semantic activity;
4. concise change summary;
5. everything else on demand.

The following should not be permanently visible in the primary execution canvas:

- complete Goal Graph;
- complete file tree;
- full diff;
- raw tool calls;
- stdout/stderr;
- runtime diagnostics;
- every validation result;
- evaluation report details;
- generic dashboard metrics.

These belong in drill-down views, drawers, inspectors, or dedicated review pages.

Whitespace is intentional. A wide desktop window does not imply that every horizontal region must contain information.

## 6. Execution Page Visual Narrative

The execution page should be a **continuous visual story**, not a collection of equally weighted cards.

The reading order is:

```text
Mission
  -> Current Goal
      -> Current Work
          -> Position in Plan
              -> Recent Activity
                  -> Change Summary
```

The visual relationships should be communicated primarily through vertical rhythm, alignment, proximity, typography, and a small number of lines/markers.

### 6.1 Mission header

The application chrome should remain quiet.

Preferred structure:

```text
<-  Add GitHub Models support                              ...
```

Avoid duplicating execution status in both the toolbar and page body. Pause/stop/change-plan actions may live in the toolbar or near the active work state, but they should not compete with the Goal itself.

### 6.2 Current Goal

The current Goal provides context, but it is not itself a dashboard card.

Example hierarchy:

```text
CURRENT GOAL
实现模型获取与缓存
从 GitHub Models 获取可用模型，并支持本地缓存与刷新机制。
```

The Goal title should be one of the strongest typographic elements on the page.

### 6.3 Current Work is the active focal point

The most dynamic and immediate information is what the executor is doing **right now**.

Example:

```text
CURRENT WORK
● 实现缓存刷新逻辑
  实现按 TTL 和条件触发的缓存刷新，保证模型列表及时更新。
```

The current marker uses the Nord Frost family, normally Nord10 for sufficient contrast. A subtle pulse or state transition may communicate activity; avoid glowing or looping decorative animation.

Do not show uncertain percentage estimates or invented time-to-completion values. Use real state only.

### 6.4 Mission path

The execution page should show where the current Goal sits in the broader Mission without keeping the entire Goal Tree open.

Preferred compressed projection:

```text
✓ 架构理解 —— ✓ 凭证与鉴权 —— ● 模型管理 —— ○ 运行时集成 —— ○ 验证 —— ○ 交付
                                3 / 7
```

Completed nodes use restrained success semantics. The current node is the strongest accent. Pending nodes remain quiet neutral outlines.

The path is contextual navigation, not a progress percentage.

A full Goal Tree / DAG projection opens only when the user requests `Plan` or a comparable disclosure action.

### 6.5 Recent Activity

Recent Activity should visually continue the active-work narrative instead of appearing as a disconnected log card.

Example:

```text
RECENT ACTIVITY

10:42   获取到 32 个模型
        GitHub Models API

10:44   完成缓存策略
        TTL 与失效规则

10:46   12 个测试通过
        cargo test

10:48   ● 正在检查 stale cache 行为
```

Rules:

- default to approximately 4–6 recent semantic events;
- event titles use normal UI typography;
- paths, commands, SHAs, counts, and machine facts may use monospace;
- newest/current event receives the strongest visual marker;
- older events fade into neutral hierarchy;
- `View all activity` opens history;
- raw logs sit one or more disclosure levels below semantic activity.

New activity may enter with a restrained opacity + small vertical translation transition. Motion indicates new state; it does not perform personality.

### 6.6 Changes as secondary disclosure

Changes are important but should not form an equal permanent column beside current execution.

Default presentation should be a compact bottom shelf or contextual action:

```text
3 files changed    +184  -21    cache.rs  github.rs  provider.rs             View changes >
```

The shelf may use a light material surface with a larger radius because it is a discrete contextual layer. Opening it reveals an inspector or dedicated Artifact/Change review surface.

The primary Execution page should not render a full diff by default.

## 7. Navigation

A full persistent 220–260 px application sidebar is not required during focused execution.

Preferred direction:

- compact navigation rail or collapsible sidebar;
- strong focus on Mission content after execution begins;
- Plan, Evaluations, Changes, Delivery, and Settings remain accessible but visually secondary;
- selection uses a soft Nord Frost tint rather than a saturated colored block.

The navigation itself may use translucent material because it is an application-level chrome layer separate from the continuous content canvas.

## 8. Typography

The typography system should express a useful distinction:

> Human intent uses the UI sans; machine facts use monospace.

Use platform-native or high-quality native sans-serif stacks for:

- Mission/Goal titles;
- descriptions;
- activity summaries;
- acceptance language;
- Delivery summaries.

Use monospace for:

- paths;
- code;
- commands;
- SHAs/revisions;
- Goal/criterion IDs where shown;
- structured diagnostic values.

Do not turn the complete application into a monospace IDE aesthetic.

Suggested starting scale for the Execution page:

```text
Mission header                 15–17 px
Goal title                     28–32 px
Current Work title             18–21 px
Body / activity                14–15 px
Metadata / timestamp           12–13 px
```

Line height should be relaxed enough for daily use, generally around 1.4–1.55 for body text.

Typography and layout must be validated in both initial locales (`zh-CN` and `en-US`). Chinese and English may not use identical glyph metrics, wrapping, or label lengths; the hierarchy above must remain intact without per-language page forks. See `I18N.md` for the full localization, CJK, IME, and layout contract.

## 9. Borders, Dividers, And Shadows

Light interfaces can easily become grids of rectangles. Talos should use borders sparingly.

Priority order for creating hierarchy:

1. whitespace;
2. typography;
3. alignment/proximity;
4. subtle surface difference;
5. divider;
6. border/shadow only when necessary.

Recommended behavior:

- continuous content areas do not need outer borders;
- use a divider to separate application chrome from Mission content;
- material shelves/drawers may use a low-contrast one-pixel edge;
- shadows should be diffuse and low-opacity;
- avoid strong gray boxes around Current Goal and Recent Activity.

## 10. Status Language

Status should first be communicated by shape/icon + label; color reinforces meaning.

Suggested semantic vocabulary:

```text
○ Pending / Ready
● Current / Running
✓ Completed / Passed
! Blocked / Needs Input
× Failed
◇ Evaluating / Judgment
```

Do not fill the page with colored status badges such as `IN PROGRESS`, `COMPLETED`, and `PENDING` on every row.

For evaluation-related states, Nord15 is available as a distinct semantic family without creating a new arbitrary product color.

Visible status labels are localized presentation; the underlying domain state remains locale-neutral. Do not use localized status text as enum, protocol, persistence, or command identity.

## 11. Motion

Motion should express state transition and causality.

Recommended starting behavior:

- 150–220 ms for common transitions;
- opacity + small translation for arriving activity;
- subtle expand/collapse for disclosure;
- restrained state-marker pulse for currently executing work;
- panel/drawer transitions should feel physically connected to the source action;
- honor reduced-motion settings.

Avoid:

- bouncing;
- glowing halos;
- animated gradients;
- persistent decorative particles;
- full-layout animation for large lists;
- fake “AI thinking” orbs.

## 12. Reference Image

![Talos Desktop execution visual reference](reference-execution-light.webp)

The image is a **directional reference**, not a pixel-perfect specification. It captures the desired overall qualities:

- light native workspace;
- generous whitespace;
- large-radius restrained material;
- one primary execution narrative;
- Goal and current work as visual focus;
- compressed plan position;
- semantic activity rather than raw logs;
- changes demoted to a compact secondary shelf.

Where the image conflicts with this document, **this document is authoritative**. In particular, the final implementation should use the Nord-derived semantic palette defined above rather than treating any illustrative orange accent in a reference mockup as canonical.

The image uses illustrative mixed Chinese/English copy to show hierarchy. It must not be treated as
one complete locale, as localization coverage, or as evidence that the layout is valid for both
initial Desktop locales. The first GPUI visual spike must validate separate, complete `zh-CN` and
`en-US` layouts. Its sidebar width is also illustrative: implementation must retain the compact or
collapsible navigation behavior required by section 7.

## 13. Initial GPUI Design Validation

Before a broad Desktop implementation proceeds, the first GPUI visual spike should validate the Execution page at realistic desktop sizes.

It should demonstrate:

- light Snow Storm / Polar Night base theme;
- Nord Frost/Aurora semantic states;
- full UI localization coverage for the selected slice in Simplified Chinese (`zh-CN`) and English (`en-US`);
- text rendering, wrapping, and mixed CJK/Latin alignment for English and Chinese;
- Chinese IME behavior for editable controls in scope;
- system/explicit locale selection and deterministic fallback behavior;
- Current Goal and Current Work hierarchy;
- compact Mission path;
- recent semantic activity stream;
- compact change shelf;
- collapsible/compact navigation;
- large-radius material with fallback when transparency/blur is unavailable;
- comfortable behavior at normal laptop and wide-monitor widths;
- reduced-motion behavior.

The spike should specifically reject regressions toward:

- dense dashboard layouts;
- permanent three-column execution views;
- nested card stacks;
- raw transcript/tool-log dominance;
- fake progress percentages;
- decorative gradients;
- hard-coded single-language view strings;
- layouts tuned only to Chinese or only to English string widths.

## 14. Scope Boundary

This document records visual intent only. It does not select a concrete GPUI implementation, theme engine, localization library, font bundle, or packaging mechanism, and it does not authorize Desktop code.

Internationalization requirements for the first Desktop implementation are authoritative in `I18N.md`; this visual document only records the layout and typography consequences.
