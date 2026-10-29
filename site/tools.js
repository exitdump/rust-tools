// The tools shown on the website.
// Adding a tool? Copy the meow entry and change the text.
// status: "stable" shows download buttons, "planned" shows a "build it" card.
window.TOOLS = [
  {
    name: "meow",
    like: "cat",
    status: "stable",
    tagline: "Print files beautifully",
    description:
      "Renders Markdown, highlights code and colors logs right in your terminal. Pipe it, and it's plain cat again.",
    features: [
      "Markdown with headings, lists, tables and code blocks",
      "Syntax highlighting for dozens of languages",
      "Colored ERROR / WARN / INFO, URLs and dates in logs",
      "Plain output when piped, so scripts keep working",
    ],
    examples: ["meow README.md", "meow app.log", "meow -p notes.md | grep TODO"],
  },
  {
    name: "count",
    like: "wc",
    status: "planned",
    tagline: "Count lines, words and characters",
    description: "A friendly wc that counts Bangla and emoji correctly and shows totals as a table.",
  },
  {
    name: "tree",
    like: "tree",
    status: "planned",
    tagline: "See a folder at a glance",
    description: "Shows a folder as a colored tree, respects .gitignore, and shows file sizes.",
  },
  {
    name: "serve",
    like: "python -m http.server",
    status: "planned",
    tagline: "Share a folder over HTTP",
    description: "One command turns the current folder into a local website, with live reload.",
  },
];
