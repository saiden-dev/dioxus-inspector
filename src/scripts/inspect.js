return (() => {
    const selector = {SELECTOR};
    const el = document.querySelector(selector);

    if (!el) {
        return JSON.stringify({
            found: false,
            selector: selector,
            error: "Element not found"
        });
    }

    const rect = el.getBoundingClientRect();
    const style = getComputedStyle(el);
    const viewport = { width: window.innerWidth, height: window.innerHeight };
    const issues = [];

    if (rect.top >= viewport.height) {
        issues.push({ type: "out_of_viewport", message: `Element below viewport (top: ${rect.top}px)` });
    }
    if (rect.bottom <= 0) {
        issues.push({ type: "out_of_viewport", message: `Element above viewport (bottom: ${rect.bottom}px)` });
    }
    if (rect.left >= viewport.width) {
        issues.push({ type: "out_of_viewport", message: `Element right of viewport (left: ${rect.left}px)` });
    }
    if (rect.right <= 0) {
        issues.push({ type: "out_of_viewport", message: `Element left of viewport (right: ${rect.right}px)` });
    }

    if (style.display === "none") {
        issues.push({ type: "display_none", message: "Element has display: none" });
    }
    if (style.visibility === "hidden") {
        issues.push({ type: "visibility_hidden", message: "Element has visibility: hidden" });
    }
    if (style.opacity === "0") {
        issues.push({ type: "opacity_zero", message: "Element has opacity: 0" });
    }
    if (rect.width === 0 || rect.height === 0) {
        issues.push({ type: "zero_dimensions", message: `Zero dimensions: ${rect.width}x${rect.height}` });
    }

    const classes = el.className.split(/\s+/).filter(c => c);
    const missingClasses = [];
    for (const cls of classes) {
        let found = false;
        for (const sheet of document.styleSheets) {
            try {
                for (const rule of sheet.cssRules || []) {
                    if (rule.selectorText && rule.selectorText.includes("." + cls)) {
                        found = true;
                        break;
                    }
                }
            } catch (e) {}
            if (found) break;
        }
        if (!found) missingClasses.push(cls);
    }

    if (missingClasses.length > 0) {
        issues.push({
            type: "css_classes_missing",
            message: `Missing classes: ${missingClasses.join(", ")}`,
            classes: missingClasses
        });
    }

    const isVisible = !issues.some(i =>
        ["out_of_viewport", "display_none", "visibility_hidden", "opacity_zero", "zero_dimensions"].includes(i.type)
    );

    return JSON.stringify({
        found: true,
        visible: isVisible,
        selector: selector,
        element: {
            tag: el.tagName.toLowerCase(),
            id: el.id || null,
            classes: classes,
            boundingRect: { top: rect.top, left: rect.left, width: rect.width, height: rect.height },
            computedStyle: {
                position: style.position,
                display: style.display,
                visibility: style.visibility,
                opacity: style.opacity,
                zIndex: style.zIndex
            }
        },
        viewport: viewport,
        issues: issues,
        summary: issues.length === 0 ? "Element is visible" : issues.map(i => i.message).join("; ")
    });
})()
