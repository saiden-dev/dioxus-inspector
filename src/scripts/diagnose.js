return (() => {
    const viewport = { width: window.innerWidth, height: window.innerHeight };
    const issues = {
        elementsOutsideViewport: [],
        elementsWithZeroDimensions: []
    };

    // Tailwind-like class patterns (JIT classes that won't be in stylesheets)
    const TAILWIND_PATTERNS = [
        /^-?[a-z]+-\[.+\]$/,  // arbitrary values: w-[100px], text-[#fff]
        /^[a-z]+-[a-z]+-\d+$/, // color scales: text-blue-500, bg-red-100
        /^(w|h|p|m|gap|text|bg|border|rounded|flex|grid|col|row|space|min|max)-/,
        /^(hover|focus|active|disabled|dark|sm|md|lg|xl|2xl):/,
        /^(font|leading|tracking|opacity|z|order|inset|top|right|bottom|left)-/
    ];

    function isTailwindClass(cls) {
        return TAILWIND_PATTERNS.some(p => p.test(cls));
    }

    const elements = document.querySelectorAll("[class]");
    for (const el of elements) {
        const rect = el.getBoundingClientRect();
        const style = getComputedStyle(el);
        const selector = el.id ? `#${el.id}` :
            (el.className ? `.${el.className.split(" ")[0]}` : el.tagName.toLowerCase());

        // Check for positioned elements outside viewport
        if (style.position === "fixed" || style.position === "absolute") {
            if (rect.top >= viewport.height || rect.bottom <= 0 ||
                rect.left >= viewport.width || rect.right <= 0) {
                issues.elementsOutsideViewport.push({
                    selector: selector,
                    position: style.position,
                    rect: { top: Math.round(rect.top), left: Math.round(rect.left), width: Math.round(rect.width), height: Math.round(rect.height) }
                });
            }
        }

        // Check for zero-dimension elements with content
        if (style.display !== "none" && style.visibility !== "hidden" && (rect.width === 0 || rect.height === 0)) {
            if (el.children.length > 0 || el.textContent.trim()) {
                issues.elementsWithZeroDimensions.push({
                    selector: selector,
                    dimensions: `${Math.round(rect.width)}x${Math.round(rect.height)}`
                });
            }
        }
    }

    // Collect z-index stack for debugging overlaps
    const zIndexStack = [];
    for (const el of document.querySelectorAll("*")) {
        const style = getComputedStyle(el);
        if ((style.position === "fixed" || style.position === "absolute") && style.zIndex !== "auto") {
            const selector = el.id ? `#${el.id}` :
                (el.className ? `.${el.className.split(" ")[0]}` : el.tagName.toLowerCase());
            zIndexStack.push({
                selector: selector,
                zIndex: parseInt(style.zIndex) || 0,
                position: style.position
            });
        }
    }
    zIndexStack.sort((a, b) => b.zIndex - a.zIndex);

    const hasIssues = issues.elementsOutsideViewport.length > 0 ||
        issues.elementsWithZeroDimensions.length > 0;

    return JSON.stringify({
        healthy: !hasIssues,
        viewport: viewport,
        issues: issues,
        zIndexStack: zIndexStack.slice(0, 10),
        elementCount: document.querySelectorAll("*").length,
        summary: hasIssues
            ? `Issues: ${issues.elementsOutsideViewport.length} outside viewport, ${issues.elementsWithZeroDimensions.length} zero-dimension`
            : "No issues detected"
    });
})()
