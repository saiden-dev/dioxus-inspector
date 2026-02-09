return (() => {
    const viewport = { width: window.innerWidth, height: window.innerHeight };
    const issues = {
        elementsOutsideViewport: [],
        elementsWithZeroDimensions: [],
        elementsWithMissingClasses: []
    };

    const checkedClasses = new Set();
    const missingClasses = new Set();
    const availableClasses = new Set();

    for (const sheet of document.styleSheets) {
        try {
            for (const rule of sheet.cssRules || []) {
                if (rule.selectorText) {
                    const matches = rule.selectorText.match(/\.[\w-\[\]\\]+/g) || [];
                    for (const match of matches) {
                        availableClasses.add(match.slice(1).replace(/\\/g, ""));
                    }
                }
            }
        } catch (e) {}
    }

    const elements = document.querySelectorAll("[class]");
    for (const el of elements) {
        const rect = el.getBoundingClientRect();
        const style = getComputedStyle(el);
        const selector = el.id ? `#${el.id}` :
            (el.className ? `.${el.className.split(" ")[0]}` : el.tagName.toLowerCase());

        if (style.position === "fixed" || style.position === "absolute") {
            if (rect.top >= viewport.height || rect.bottom <= 0 ||
                rect.left >= viewport.width || rect.right <= 0) {
                issues.elementsOutsideViewport.push({
                    selector: selector,
                    position: style.position,
                    rect: { top: rect.top, left: rect.left, width: rect.width, height: rect.height }
                });
            }
        }

        if (style.display !== "none" && (rect.width === 0 || rect.height === 0)) {
            if (el.children.length > 0 || el.textContent.trim()) {
                issues.elementsWithZeroDimensions.push({
                    selector: selector,
                    dimensions: `${rect.width}x${rect.height}`
                });
            }
        }

        const classes = el.className.split(/\s+/).filter(c => c);
        for (const cls of classes) {
            if (!checkedClasses.has(cls)) {
                checkedClasses.add(cls);
                if (!availableClasses.has(cls)) {
                    missingClasses.add(cls);
                }
            }
        }
    }

    if (missingClasses.size > 0) {
        issues.elementsWithMissingClasses = Array.from(missingClasses);
    }

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
        issues.elementsWithZeroDimensions.length > 0 ||
        issues.elementsWithMissingClasses.length > 0;

    return JSON.stringify({
        healthy: !hasIssues,
        viewport: viewport,
        issues: issues,
        zIndexStack: zIndexStack.slice(0, 10),
        summary: hasIssues
            ? `Issues: ${issues.elementsOutsideViewport.length} outside viewport, ${issues.elementsWithMissingClasses.length} missing CSS`
            : "No issues detected"
    });
})()
