return (() => {
    function serialize(el, depth = 0) {
        if (depth > 10) return { tag: '...', truncated: true };
        if (el.nodeType === 3) {
            const text = el.textContent.trim();
            return text ? { text } : null;
        }
        if (el.nodeType !== 1) return null;

        const node = { tag: el.tagName.toLowerCase() };
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
    return JSON.stringify(serialize(document.body), null, 2);
})()
