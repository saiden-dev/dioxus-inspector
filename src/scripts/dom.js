return (() => {
    const SKIP_TAGS = new Set(['script', 'style', 'noscript', 'link', 'meta']);
    const MAX_TEXT_LEN = 100;
    const MAX_DEPTH = {MAX_DEPTH};
    const MAX_NODES = {MAX_NODES};
    const SELECTOR = {SELECTOR};

    let nodeCount = 0;
    let truncatedByLimit = false;

    function serialize(el, depth = 0) {
        if (nodeCount >= MAX_NODES) {
            truncatedByLimit = true;
            return null;
        }

        if (depth > MAX_DEPTH) return { tag: '...', truncated: 'depth' };

        if (el.nodeType === 3) {
            let text = el.textContent.trim();
            if (!text) return null;
            nodeCount++;
            if (text.length > MAX_TEXT_LEN) {
                text = text.slice(0, MAX_TEXT_LEN) + '...';
            }
            return { text };
        }

        if (el.nodeType !== 1) return null;

        const tag = el.tagName.toLowerCase();
        if (SKIP_TAGS.has(tag)) return null;

        nodeCount++;
        const node = { tag };
        if (el.id) node.id = el.id;
        if (el.className && typeof el.className === 'string') {
            node.class = el.className;
        }

        const children = [];
        for (const child of el.childNodes) {
            if (nodeCount >= MAX_NODES) {
                children.push({ tag: '...', truncated: 'max_nodes', remaining: el.childNodes.length - children.length });
                break;
            }
            const serialized = serialize(child, depth + 1);
            if (serialized) children.push(serialized);
        }
        if (children.length > 0) node.children = children;

        return node;
    }

    const root = SELECTOR ? document.querySelector(SELECTOR) : document.body;
    if (!root) return JSON.stringify({ error: 'Selector not found: ' + SELECTOR });

    const tree = serialize(root);
    return JSON.stringify({
        root: tree,
        stats: { nodeCount, maxNodes: MAX_NODES, maxDepth: MAX_DEPTH, truncated: truncatedByLimit }
    });
})()
