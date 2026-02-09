return (() => {
    const SKIP_TAGS = new Set(['script', 'style', 'noscript', 'link', 'meta']);
    const MAX_TEXT_LEN = 80;
    const MAX_CLASS_LEN = 40;

    function serialize(el, depth = 0, indent = '') {
        if (depth > 10) return indent + '...\n';

        // Text nodes
        if (el.nodeType === 3) {
            let text = el.textContent.trim();
            if (!text) return '';
            if (text.length > MAX_TEXT_LEN) {
                text = text.slice(0, MAX_TEXT_LEN) + '...';
            }
            return indent + '"' + text + '"\n';
        }

        if (el.nodeType !== 1) return '';

        const tag = el.tagName.toLowerCase();
        if (SKIP_TAGS.has(tag)) return '';

        // Build selector-like representation
        let line = tag;
        if (el.id) line += '#' + el.id;
        if (el.className && typeof el.className === 'string') {
            let classes = el.className.trim();
            if (classes.length > MAX_CLASS_LEN) {
                classes = classes.slice(0, MAX_CLASS_LEN) + '...';
            }
            if (classes) line += '.' + classes.split(/\s+/).join('.');
        }

        let result = indent + line + '\n';

        // Recurse children
        for (const child of el.childNodes) {
            result += serialize(child, depth + 1, indent + '  ');
        }

        return result;
    }

    return serialize(document.body).trim();
})()
