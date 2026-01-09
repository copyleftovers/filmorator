pub const CSS_RESET: &str = "* { box-sizing: border-box; margin: 0; padding: 0; }";

pub const CSS_VARS: &str = r"
:root {
    --bg: #1a1a1a;
    --fg: #f0f0f0;
    --muted: #888;
    --font: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}
body {
    font-family: var(--font);
    background: var(--bg);
    color: var(--fg);
}
a { color: var(--fg); }
";
