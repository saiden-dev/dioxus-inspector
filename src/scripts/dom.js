return (() => {
    const SKIP_TAGS = new Set(['script', 'style', 'noscript', 'link', 'meta']);
    const MAX_TEXT_LEN = 200;

    function serialize(el, depth = 0) {
        if (depth > 10) return { tag: '...', truncated: true };

        if (el.nodeType === 3) {
            let text = el.textContent.trim();
            if (!text) return null;
            if (text.length > MAX_TEXT_LEN) {
                text = text.slice(0, MAX_TEXT_LEN) + '...';
            }
            return { text };
        }

        if (el.nodeType !== 1) return null;

        const tag = el.tagName.toLowerCase();
        if (SKIP_TAGS.has(tag)) return null;

        const node = { tag };
        if (el.id) node.id = el.id;
        if (el.className && typeof el.className === 'string') {
            node.class = el.className;
        }

        const children = [];
        for (const child of el.childNodes) {
            const serialized = serialize(child, depth + 1);
            if (serialized) children.push(serialized);
        }
        if (children.length > 0) node.children = children;

        return node;
    }
    return JSON.stringify(serialize(document.body));
})()
