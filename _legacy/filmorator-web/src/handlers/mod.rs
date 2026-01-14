pub mod api;
pub mod compare;
pub mod session;
mod style;

use axum::response::Html;

pub async fn index() -> Html<String> {
    Html(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Filmorator</title>
    <style>
{css_reset}
{css_vars}
body {{
    min-height: 100vh;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
    text-align: center;
}}
h1 {{ font-size: 3rem; font-weight: 300; margin-bottom: 1rem; }}
p {{ color: var(--muted); margin-bottom: 2rem; }}
.btn {{
    display: inline-block;
    padding: 1rem 2rem;
    background: var(--fg);
    color: var(--bg);
    text-decoration: none;
    border-radius: 8px;
}}
    </style>
</head>
<body>
    <main>
        <h1>Filmorator</h1>
        <p>Rank film photographs through direct comparison.</p>
        <a href="/compare" class="btn">Start Comparing</a>
    </main>
</body>
</html>"#,
        css_reset = style::CSS_RESET,
        css_vars = style::CSS_VARS,
    ))
}
