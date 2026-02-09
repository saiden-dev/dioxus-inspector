return (() => {
    const classesToCheck = {CLASSES};
    const results = {};
    const availableClasses = new Set();
    const classRules = {};

    for (const sheet of document.styleSheets) {
        try {
            for (const rule of sheet.cssRules || []) {
                if (rule.selectorText) {
                    const matches = rule.selectorText.match(/\.[\w-\[\]\\]+/g) || [];
                    for (const match of matches) {
                        const cls = match.slice(1).replace(/\\/g, "");
                        availableClasses.add(cls);
                        if (!classRules[cls]) {
                            classRules[cls] = rule.cssText.substring(0, 200);
                        }
                    }
                }
            }
        } catch (e) {}
    }

    for (const cls of classesToCheck) {
        const found = availableClasses.has(cls);
        results[cls] = { found: found, rule: found ? classRules[cls] : null };
    }

    const missing = classesToCheck.filter(c => !results[c].found);
    const found = classesToCheck.filter(c => results[c].found);

    return JSON.stringify({
        results: results,
        summary: {
            total: classesToCheck.length,
            found: found.length,
            missing: missing.length,
            missingClasses: missing
        }
    });
})()
