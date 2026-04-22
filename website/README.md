# goblint Rules Documentation Website

This directory contains the static website for browsing goblint rules, deployed to GitHub Pages.

## Files

- `index.html` - Main HTML structure
- `style.css` - All CSS styles with light/dark theme support
- `script.js` - JavaScript for rendering rules, filtering, and navigation
- `rules.json` - **Generated file** (not in git) - Created by CI from `goblint --list-rules --format=json`

## Local Development

To run the website locally:

```bash
# Generate rules.json
cargo run --bin goblint -- --list-rules --format=json > website/rules.json

# Serve with any HTTP server, e.g.:
python3 -m http.server 8000 --directory website
# or
cd website && npx serve
```

Then visit http://localhost:8000

## Deployment

The website is automatically deployed to GitHub Pages via `.github/workflows/pages.yml` on every push to `main`. The workflow:

1. Builds the goblint CLI
2. Generates `rules.json` from the latest rule definitions
3. Uploads everything to GitHub Pages

**Note:** `rules.json` is excluded from git (see `.gitignore`) since it's generated during CI.
