# Claude UI - Phase 1 Refined

A clean, production-ready UI foundation using Next.js (App Router) and Tailwind CSS.

## Design System Compliance ✅

**Strict colors:**
- `background: #000000`
- `foreground: #FFFFFF`
- `muted: #A1A1AA`
- `border: #1A1A1A`

**Spacing System (8px Grid):**
- Sidebar: 260px fixed width
- Chat max width: ~720px (via `max-w-3xl` container)
- Message spacing: 24px (`space-y-6`)
- Padding: 16px-32px scale

**Typography:**
- Base +1.6 line height
- Wide letter-spacing (0.02em)
- Inter font family
- 12px-14px base text sizes

## Layout Structure

### Components (Refined)

1. **Sidebar** (left)
   - Fixed width: 260px
   - Subtle borders: `border-white/[0.06]`
   - Title: Small, tracking-tight, 14px
   - Button: Full height, hover states
   - Vertical spacing: px-4 py-6 / py-4

2. **Chat Area** (right)
   - Fill remaining space
   - Messages: Centered, max-w-3xl
   - Input: Sticky bottom, rounded-xl container

3. **Status Bar** (bottom)
   - Subtle colors: muted-foreground/70
   - Clean separators with darker opacity

## UI Primitives

All reusable components in `src/components/ui/`:
- `Button` - Outline variant, subtle borders, smooth transitions
- `Input` - Clean focus states
- `ScrollContainer` - Overflow handling

## Tech Stack

- Next.js 15 with App Router
- TypeScript
- Tailwind CSS
- React 18

## Phase 1 Features (Completed)

✅ Terminal-size-aware status bar
✅ Live token counter placeholder
✅ Turn duration timer placeholder
✅ Git branch indicator
✅ Clean, minimal design
✅ Fixed layout structure

✅ **PHASE A ENHANCEMENTS:**
- 8px grid spacing system
- Chat max width at 720px
- Sticky input with backdrop blur
- Subtle background glow (3% on focus, 6% on focus-within)
- Better message padding (px-5 py-3)
- Refined color opacity (foreground/96, white/5)
- Smaller text sizes (text-sm for sidebar)
- Subtle borders (white/[0.06])
- Custom scrollbars
- Improved focus states

## Folder Structure

```
ui/
├── src/
│   ├── app/
│   │   ├── layout.tsx       # Root layout with metadata
│   │   ├── page.tsx         # Home page
│   │   └── globals.css      # Global styles
│   ├── components/
│   │   ├── layout/          # Layout components
│   │   │   ├── index.ts
│   │   │   ├── layout.tsx
│   │   │   ├── sidebar.tsx
│   │   │   ├── chat-area.tsx
│   │   │   └── status-bar.tsx
│   │   └── ui/              # Reusable components
│   │       ├── button.tsx
│   │       ├── input.tsx
│   │       └── scroll-container.tsx
│   └── lib/                 # Utility functions (to be added)
├── tailwind.config.ts       # Tailwind configuration with new tokens
└── package.json
```

## Design Checklist

✅ Claude-style minimal aesthetic
✅ Linear-style clean typography
✅ Notion AI-inspired spacing
✅ Subtle hover states
✅ Smooth transitions (150ms-200ms)
✅ Terminal-size-aware UI
✅ Backdrop blur effects
✅ Custom scrollbar styling
✅ Color opacity usage
✅ Letter-spacing for modern look

## Usage

Run the dev server:
```bash
cd ui
npm install
npm run dev
```

Open your browser to see the refined UI.

## Success Criteria Met

✅ UI renders correctly on Claude-level aesthetic
✅ Layout matches Claude/Linear/Notion AI style
✅ Code is clean and reusable
✅ No visual inconsistencies
✅ Strict design system compliance (color opacity)
✅ Subtle interactions (hover glow, backdrop)
✅ Proportional spacing system (8px grid)
✅ Modern typography (Inter, wide spacing)
✅ No gradients or extra colors
✅ Minimal border usage (subtle opacity)

## Next Steps

- Implement message display components
- Connect to API
- Add live token counter updates
- Add turn duration timer
- Markdown rendering
- Code syntax highlighting

## Design Philosophy

This UI follows Claude's **principles**:
- **Less is more** — Only essential UI elements
- **Subtle interactions** — Smooth transitions, gentle glows
- **Clear information hierarchy** — Proper font sizes and weights
- **Black & white only** — Strict color palette
- **Consistent spacing** — 8px grid system
- **Modern typography** — Inter, proper line height and letter spacing