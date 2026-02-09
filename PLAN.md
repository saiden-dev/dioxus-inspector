# Plan: Inspector Quick Wins

## Phase 1: Window Control

### Description
Add window manipulation capabilities to enable responsive testing workflows.

### Steps

#### Step 1.1: Add resize endpoint to bridge
- **Objective**: Allow resizing the window via HTTP API
- **Files**: `src/lib.rs`, `src/handlers.rs`
- **Dependencies**: None
- **Implementation**:
  - Add `POST /resize` endpoint accepting `{ width, height }`
  - Use JavaScript `window.resizeTo()` or Dioxus window API
  - Return new window dimensions in response

#### Step 1.2: Add resize MCP tool
- **Objective**: Expose resize to Claude Code
- **Files**: `mcp-server/src/main.rs`
- **Dependencies**: Step 1.1
- **Implementation**:
  - Add `resize` tool with width/height parameters
  - Call bridge `/resize` endpoint
  - Add preset sizes: mobile (375x667), tablet (768x1024), desktop (1280x800)

## Phase 2: DOM Statistics

### Description
Add quick stats about the DOM for understanding page complexity.

### Steps

#### Step 2.1: Add stats endpoint to bridge
- **Objective**: Return element counts and tag distribution
- **Files**: `src/handlers.rs`, `src/scripts/stats.js`
- **Dependencies**: None
- **Implementation**:
  - Create `stats.js` script that counts elements by tag
  - Add `GET /stats` endpoint
  - Return: `{ total, by_tag: { div: N, button: N, ... }, depth: N }`

#### Step 2.2: Add stats MCP tool
- **Objective**: Expose stats to Claude Code
- **Files**: `mcp-server/src/main.rs`
- **Dependencies**: Step 2.1
- **Implementation**:
  - Add `stats` tool (no parameters)
  - Format output as readable summary

## Phase 3: Selector Generator

### Description
Generate unique CSS selectors for elements to use in tests or automation.

### Steps

#### Step 3.1: Add selector script
- **Objective**: Generate minimal unique CSS selector for any element
- **Files**: `src/scripts/selector.js`
- **Dependencies**: None
- **Implementation**:
  - Create script that builds selector from id, classes, nth-child
  - Prefer: `#id` > `.class` > `tag.class` > `parent > child`
  - Verify uniqueness with `querySelectorAll`

#### Step 3.2: Add selector endpoint and MCP tool
- **Objective**: Expose selector generation via API
- **Files**: `src/handlers.rs`, `mcp-server/src/main.rs`
- **Dependencies**: Step 3.1
- **Implementation**:
  - Add `POST /selector` endpoint accepting element identifier
  - Add `get_selector` MCP tool
  - Return shortest unique selector

## Phase 4: Color Extractor

### Description
Extract colors used in the UI for design consistency checks.

### Steps

#### Step 4.1: Add colors script
- **Objective**: Scan page for all colors in use
- **Files**: `src/scripts/colors.js`
- **Dependencies**: None
- **Implementation**:
  - Walk DOM, extract computed `color`, `background-color`, `border-color`
  - Deduplicate and normalize to hex
  - Group by usage type (text, background, border)

#### Step 4.2: Add colors endpoint and MCP tool
- **Objective**: Expose color extraction via API
- **Files**: `src/handlers.rs`, `mcp-server/src/main.rs`
- **Dependencies**: Step 4.1
- **Implementation**:
  - Add `GET /colors` endpoint
  - Add `colors` MCP tool
  - Return grouped color list with usage counts

## Phase 5: Font Inspector

### Description
List all fonts used in the page for typography auditing.

### Steps

#### Step 5.1: Add fonts script
- **Objective**: Extract font families and sizes in use
- **Files**: `src/scripts/fonts.js`
- **Dependencies**: None
- **Implementation**:
  - Walk DOM, extract computed `font-family`, `font-size`, `font-weight`
  - Group by font family
  - List all size/weight combinations per family

#### Step 5.2: Add fonts endpoint and MCP tool
- **Objective**: Expose font inspection via API
- **Files**: `src/handlers.rs`, `mcp-server/src/main.rs`
- **Dependencies**: Step 5.1
- **Implementation**:
  - Add `GET /fonts` endpoint
  - Add `fonts` MCP tool
  - Return font usage summary
