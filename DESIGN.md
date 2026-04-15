# Design System: Retro-Modern Pixel Tech

 

## 1. Overview & Creative North Star

 

### Creative North Star: "The Analog Architect"

The vision for this design system is to bridge the nostalgic warmth of 8-bit computing with the high-fidelity precision of modern editorial design. We are moving beyond the "clunky" retro trope to create a sophisticated, high-end experience that feels curated and intentional. 

 

This system breaks the rigid, "boxed-in" feeling of traditional tech tools through **Asymmetric Precision**. By combining pixel-inspired display elements with vast amounts of negative space and overlapping "glass" layers, we create a UI that feels like a digital canvas rather than a database. The result is a technical tool that breathes—balancing the technical sharpness of vibrant blues with the human warmth of organic skin tones.

 

---

 

## 2. Colors

 

The palette is a high-contrast dialogue between tech-forward blues and grounding, warm neutrals.

 

*   **Primary (`#0061a4`):** The "Digital Pulse." Used for high-priority actions and brand-defining moments.

*   **Secondary (`#984628`):** The "Warm Skin." Derived from the logo's pixelated skin tones, this provides a human touch and serves as a sophisticated counter-point to the cool primary blues.

*   **Tertiary (`#01658c`):** Deep lake blue for supportive accents and depth.

*   **Surface Hierarchy:** We utilize a refined grayscale of `surface_container` tokens to build structure without friction.

 

### The "No-Line" Rule

**Explicit Instruction:** Do not use 1px solid borders for sectioning content. Boundaries must be defined solely through background color shifts. For example, a `surface_container_low` section should sit directly on a `background` surface. If separation is needed, use vertical white space or a change in tonal value.

 

### The "Glass & Gradient" Rule

To elevate the experience from "flat" to "premium," floating navigation elements and modals should utilize **Glassmorphism**. Apply a semi-transparent `surface_container_lowest` with a 20px-40px backdrop-blur. 

*   **Signature Textures:** For Hero sections or primary CTAs, apply a subtle linear gradient from `primary` (`#0061a4`) to `primary_container` (`#2196f3`) at a 135-degree angle. This adds "visual soul" and a sense of light.

 

---

 

## 3. Typography

 

The typography scale is designed to create an editorial cadence, moving from "Digital/Geometrical" for impact to "Functional/Swiss" for utility.

 

*   **Display & Headline (Space Grotesk):** This represents the "Modern" in Retro-Modern. It is geometric, quirky, and digital in spirit. Use `display-lg` (3.5rem) with tight letter-spacing (-0.02em) to create an authoritative, editorial header presence.

*   **Title, Body, & Labels (Inter):** Inter provides the necessary "Functional" clarity. Its neutral stance allows the logo's pixel art and the Space Grotesk headers to take center stage without visual clutter.

*   **Hierarchy Intent:** Large, asymmetric headlines drive the user’s eye, while Inter's body text (set to `body-md` at 0.875rem) provides the technical readability required for a data-heavy tool.

 

---

 

## 4. Elevation & Depth

 

We reject traditional shadows in favor of **Tonal Layering**.

 

*   **The Layering Principle:** Depth is achieved by "stacking" surface-container tiers. Place a `surface_container_lowest` card on a `surface_container_low` background. This creates a soft, natural lift that mimics the stacking of high-quality paper.

*   **Ambient Shadows:** If a floating element (like a Tooltip or Dropdown) requires a shadow, it must be an "Ambient Shadow": 

    *   **Blur:** 32px to 64px.

    *   **Opacity:** 6% of the `on_surface` color. 

    *   **Tint:** Always tint the shadow with 2% of the `primary` color to keep the shadows from feeling "dead" or muddy.

*   **The "Ghost Border" Fallback:** If accessibility requirements demand a container edge, use a "Ghost Border": the `outline_variant` token at 15% opacity. Never use 100% opaque borders.

 

---

 

## 5. Components

 

### Buttons

*   **Primary:** `primary` background, `on_primary` text. Use `DEFAULT` roundness (0.5rem). For a signature touch, add a 2px "inner-glow" on hover using a lighter shade of blue.

*   **Secondary:** `secondary_container` background with `on_secondary_container` text. This leverages the warm logo tones to differentiate from the tech-blue actions.

 

### Cards & Lists

*   **The Forbidden Divider:** Never use horizontal lines to separate list items. Use a 12px-16px gap and alternating `surface` to `surface_container_low` background shifts for row distinction.

*   **Cards:** Use `surface_container_lowest`. Do not add borders. Use `lg` (1rem) roundness for the outer container to create a "friendly" tech aesthetic.

 

### Input Fields

*   **Style:** Minimalist. Use `surface_variant` for the background with a `sm` (0.25rem) roundness. Upon focus, transition the background to `surface_container_highest` and provide a 2px `primary` bottom-only highlight (a nod to terminal cursors).

 

### Signature Component: The "Pixel Frame"

*   For featured images or profile avatars, use a custom mask that subtly mimics the pixelated edges of the logo on just one or two corners (e.g., top-left and bottom-right), while keeping the other corners rounded at `md` (0.75rem).

 

---

 

## 6. Do's and Don'ts

 

### Do:

*   **DO** use whitespace as a functional element. Hero sections should have a minimum of 80px padding.

*   **DO** mix the warm `secondary` tones with cool `primary` blues to maintain the "AI Companion" persona—the tool should feel like a partner, not just a utility.

*   **DO** use `surface_bright` for interactive "Glass" elements to ensure they pop against the `f9f9f9` background.

 

### Don't:

*   **DON'T** use 100% black (`#000000`). Always use `on_surface` (`#1a1c1c`) for text to maintain a premium, slightly softer contrast.

*   **DON'T** use traditional "Material Design" drop shadows. Stick to the Ambient Shadow or Tonal Layering.

*   **DON'T** clutter the UI with icons. Let the typography and the pixel-art logo drive the visual interest.

*   **DON'T** use sharp 90-degree corners. The "friendly" requirement dictates a minimum of `sm` (0.25rem) roundness for all elements.