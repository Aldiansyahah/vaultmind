<script lang="ts">
  import { onDestroy } from "svelte";
  import { Editor } from "@tiptap/core";
  import StarterKit from "@tiptap/starter-kit";
  import Link from "@tiptap/extension-link";
  import { selectedNotePath, noteContent, isLoading, vaultEntries } from "$lib/stores/vault";
  import { saveNoteContent } from "$lib/stores/vault-actions";
  import WikilinkAutocomplete from "$lib/components/WikilinkAutocomplete.svelte";
  import TagAutocomplete from "$lib/components/TagAutocomplete.svelte";

  let editor: Editor | null = null;
  let container = $state<HTMLElement | null>(null);
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;
  let isInternalUpdate = false;

  let showAutocomplete = $state(false);
  let autocompleteType = $state<"wikilink" | "tag">("wikilink");
  let autocompleteQuery = $state("");
  let selectedIndex = $state(0);
  let autocompletePosition = $state({ top: 0, left: 0 });

  const existingTags = ["rust", "programming", "notes", "todo", "idea", "reference"];

  /// Converts basic Markdown to HTML for TipTap editor input.
  function markdownToHtml(md: string): string {
    return md
      .split("\n\n")
      .map((block) => {
        const trimmed = block.trim();
        if (!trimmed) return "";
        if (trimmed.startsWith("### ")) return `<h3>${trimmed.slice(4)}</h3>`;
        if (trimmed.startsWith("## ")) return `<h2>${trimmed.slice(3)}</h2>`;
        if (trimmed.startsWith("# ")) return `<h1>${trimmed.slice(2)}</h1>`;
        if (trimmed.startsWith("```")) {
          const lines = trimmed.split("\n");
          const code = lines.slice(1, -1).join("\n");
          return `<pre><code>${code}</code></pre>`;
        }
        if (trimmed.startsWith("> ")) {
          const quote = trimmed
            .split("\n")
            .map((l) => l.replace(/^>\s?/, ""))
            .join("<br>");
          return `<blockquote><p>${quote}</p></blockquote>`;
        }
        if (trimmed.startsWith("- ") || trimmed.startsWith("* ")) {
          const items = trimmed
            .split("\n")
            .map((l) => `<li>${l.replace(/^[-*]\s/, "")}</li>`)
            .join("");
          return `<ul>${items}</ul>`;
        }
        if (/^\d+\.\s/.test(trimmed)) {
          const items = trimmed
            .split("\n")
            .map((l) => `<li>${l.replace(/^\d+\.\s/, "")}</li>`)
            .join("");
          return `<ol>${items}</ol>`;
        }
        // Inline formatting
        let html = trimmed
          .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
          .replace(/\*(.+?)\*/g, "<em>$1</em>")
          .replace(/`([^`]+)`/g, "<code>$1</code>")
          .replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2">$1</a>')
          .replace(/\n/g, "<br>");
        return `<p>${html}</p>`;
      })
      .filter(Boolean)
      .join("");
  }

  /// Converts TipTap HTML output to Markdown for saving to disk.
  function htmlToMarkdown(html: string): string {
    const div = document.createElement("div");
    div.innerHTML = html;

    function processNode(node: Node): string {
      if (node.nodeType === Node.TEXT_NODE) {
        return node.textContent || "";
      }

      if (node.nodeType !== Node.ELEMENT_NODE) return "";

      const el = node as HTMLElement;
      const tag = el.tagName.toLowerCase();
      const childText = () => Array.from(el.childNodes).map(processNode).join("");

      switch (tag) {
        case "h1":
          return `# ${childText()}\n\n`;
        case "h2":
          return `## ${childText()}\n\n`;
        case "h3":
          return `### ${childText()}\n\n`;
        case "p":
          return `${childText()}\n\n`;
        case "strong":
        case "b":
          return `**${childText()}**`;
        case "em":
        case "i":
          return `*${childText()}*`;
        case "code":
          if (el.parentElement?.tagName.toLowerCase() === "pre") return childText();
          return `\`${childText()}\``;
        case "pre":
          return `\`\`\`\n${childText()}\n\`\`\`\n\n`;
        case "blockquote":
          return (
            childText()
              .trim()
              .split("\n")
              .map((line: string) => `> ${line}`)
              .join("\n") + "\n\n"
          );
        case "ul":
          return (
            Array.from(el.children)
              .map((li) => `- ${processNode(li).trim()}`)
              .join("\n") + "\n\n"
          );
        case "ol":
          return (
            Array.from(el.children)
              .map((li, i) => `${i + 1}. ${processNode(li).trim()}`)
              .join("\n") + "\n\n"
          );
        case "li":
          return childText();
        case "a": {
          const href = el.getAttribute("href") || "";
          return `[${childText()}](${href})`;
        }
        case "br":
          return "\n";
        default:
          return childText();
      }
    }

    return (
      Array.from(div.childNodes)
        .map(processNode)
        .join("")
        .replace(/\n{3,}/g, "\n\n")
        .trim() + "\n"
    );
  }

  function getNoteTitles(): string[] {
    const titles: string[] = [];
    function extractTitles(entries: typeof $vaultEntries) {
      for (const entry of entries) {
        if (!entry.is_directory) {
          titles.push(entry.name.replace(/\.md$/, ""));
        }
        if (entry.children) {
          extractTitles(entry.children);
        }
      }
    }
    extractTitles($vaultEntries);
    return titles;
  }

  function getWikilinkSuggestions(query: string): string[] {
    if (!query) return getNoteTitles().slice(0, 10);
    const titles = getNoteTitles();
    return titles.filter((t) => t.toLowerCase().includes(query.toLowerCase())).slice(0, 10);
  }

  function getTagSuggestions(query: string): string[] {
    if (!query) return existingTags.slice(0, 10);
    return existingTags.filter((t) => t.toLowerCase().includes(query.toLowerCase())).slice(0, 10);
  }

  function handleWikilinkSelect(target: string) {
    if (!editor) return;
    const sel = editor.state.selection;
    const pos = sel.$from.pos;
    const parentOffset = sel.$from.parentOffset;
    const textBefore = sel.$from.parent.textBetween(
      Math.max(0, parentOffset - 2),
      parentOffset,
      undefined,
      " ",
    );

    const queryMatch = textBefore.match(/\[\[([^\]]*)$/);
    if (queryMatch) {
      const queryStart = pos - queryMatch[0].length;
      const queryEnd = pos;
      editor
        .chain()
        .focus()
        .deleteRange({ from: queryStart, to: queryEnd })
        .insertContent(`[[${target}]] `)
        .run();
    }

    showAutocomplete = false;
    autocompleteQuery = "";
  }

  function handleTagSelect(tag: string) {
    if (!editor) return;
    const sel = editor.state.selection;
    const pos = sel.$from.pos;
    const parentOffset = sel.$from.parentOffset;
    const textBefore = sel.$from.parent.textBetween(
      Math.max(0, parentOffset - 1),
      parentOffset,
      undefined,
      " ",
    );

    const queryMatch = textBefore.match(/#([a-zA-Z0-9_-]*)$/);
    if (queryMatch) {
      const queryStart = pos - queryMatch[0].length;
      const queryEnd = pos;
      editor
        .chain()
        .focus()
        .deleteRange({ from: queryStart, to: queryEnd })
        .insertContent(`#${tag} `)
        .run();
    }

    showAutocomplete = false;
    autocompleteQuery = "";
  }

  function checkForAutocompleteTrigger() {
    if (!editor) return;
    const sel = editor.state.selection;
    const pos = sel.$from.pos;
    const parentOffset = sel.$from.parentOffset;
    const textBefore = sel.$from.parent.textBetween(
      Math.max(0, parentOffset - 50),
      parentOffset,
      undefined,
      " ",
    );

    const wikilinkMatch = textBefore.match(/\[\[([^\]]*)$/);
    if (wikilinkMatch) {
      autocompleteQuery = wikilinkMatch[1];
      autocompleteType = "wikilink";
      const suggestions = getWikilinkSuggestions(autocompleteQuery);
      if (suggestions.length > 0) {
        showAutocomplete = true;
        selectedIndex = 0;
        const coords = editor.view.coordsAtPos(pos);
        autocompletePosition = { top: coords.bottom + 5, left: coords.left };
      } else {
        showAutocomplete = false;
      }
      return;
    }

    const tagMatch = textBefore.match(/#([a-zA-Z0-9_-]*)$/);
    if (tagMatch) {
      autocompleteQuery = tagMatch[1];
      autocompleteType = "tag";
      const suggestions = getTagSuggestions(autocompleteQuery);
      if (suggestions.length > 0) {
        showAutocomplete = true;
        selectedIndex = 0;
        const coords = editor.view.coordsAtPos(pos);
        autocompletePosition = { top: coords.bottom + 5, left: coords.left };
      } else {
        showAutocomplete = false;
      }
      return;
    }

    showAutocomplete = false;
  }

  function createEditor(el: HTMLElement) {
    editor?.destroy();
    const initialHtml = markdownToHtml($noteContent);
    editor = new Editor({
      element: el,
      extensions: [
        StarterKit.configure({
          heading: { levels: [1, 2, 3] },
        }),
        Link.configure({ openOnClick: false }),
      ],
      content: initialHtml,
      onUpdate: () => {
        if (isInternalUpdate || !editor) return;
        const html = editor.getHTML();
        const markdown = htmlToMarkdown(html);
        checkForAutocompleteTrigger();
        if (saveTimeout) clearTimeout(saveTimeout);
        saveTimeout = setTimeout(() => {
          const path = $selectedNotePath;
          if (path) {
            saveNoteContent(path, markdown);
          }
        }, 1000);
      },
      onSelectionUpdate: () => {
        checkForAutocompleteTrigger();
      },
    });
  }

  $effect(() => {
    if (container) {
      createEditor(container);
    }
    return () => {
      editor?.destroy();
      editor = null;
    };
  });

  onDestroy(() => {
    editor?.destroy();
    editor = null;
  });

  $effect(() => {
    const content = $noteContent;
    if (editor && !editor.isDestroyed) {
      const html = markdownToHtml(content);
      const currentHtml = editor.getHTML();
      if (html !== currentHtml) {
        isInternalUpdate = true;
        editor.commands.setContent(html);
        isInternalUpdate = false;
      }
    }
  });

  $effect(() => {
    if ($isLoading && editor && !editor.isDestroyed) {
      editor.commands.clearContent();
    }
  });
</script>

<div class="editor-container">
  {#if $isLoading}
    <div class="loading">Loading...</div>
  {:else if $selectedNotePath}
    <div class="editor-wrapper" bind:this={container}></div>
  {:else}
    <div class="placeholder">
      <p>Select a note from the file tree or create a new one.</p>
    </div>
  {/if}

  {#if autocompleteType === "wikilink"}
    <WikilinkAutocomplete
      suggestions={getWikilinkSuggestions(autocompleteQuery)}
      {selectedIndex}
      visible={showAutocomplete}
      position={autocompletePosition}
      on:select={(e) => handleWikilinkSelect(e.detail)}
      on:close={() => (showAutocomplete = false)}
    />
  {:else}
    <TagAutocomplete
      tags={getTagSuggestions(autocompleteQuery)}
      {selectedIndex}
      visible={showAutocomplete}
      position={autocompletePosition}
      on:select={(e) => handleTagSelect(e.detail)}
      on:close={() => (showAutocomplete = false)}
    />
  {/if}
</div>

<style>
  .editor-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-primary);
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-secondary);
  }

  .editor-wrapper {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }

  .editor-wrapper :global(.ProseMirror) {
    outline: none;
    min-height: 100%;
    color: var(--text-primary);
    font-size: 0.95rem;
    line-height: 1.6;
  }

  .editor-wrapper :global(.ProseMirror p) {
    margin: 0.5rem 0;
  }

  .editor-wrapper :global(.ProseMirror h1),
  .editor-wrapper :global(.ProseMirror h2),
  .editor-wrapper :global(.ProseMirror h3) {
    margin: 1rem 0 0.5rem;
    font-weight: 600;
  }

  .editor-wrapper :global(.ProseMirror h1) {
    font-size: 1.75rem;
  }

  .editor-wrapper :global(.ProseMirror h2) {
    font-size: 1.4rem;
  }

  .editor-wrapper :global(.ProseMirror h3) {
    font-size: 1.15rem;
  }

  .editor-wrapper :global(.ProseMirror code) {
    background: var(--bg-secondary);
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-family: "JetBrains Mono", "Fira Code", monospace;
    font-size: 0.9em;
  }

  .editor-wrapper :global(.ProseMirror pre) {
    background: var(--bg-secondary);
    padding: 1rem;
    border-radius: 6px;
    overflow-x: auto;
  }

  .editor-wrapper :global(.ProseMirror pre code) {
    background: none;
    padding: 0;
  }

  .editor-wrapper :global(.ProseMirror blockquote) {
    border-left: 3px solid var(--accent-color);
    padding-left: 1rem;
    margin: 0.5rem 0;
    color: var(--text-secondary);
  }

  .editor-wrapper :global(.ProseMirror ul),
  .editor-wrapper :global(.ProseMirror ol) {
    padding-left: 1.5rem;
    margin: 0.5rem 0;
  }

  .editor-wrapper :global(.ProseMirror a) {
    color: var(--accent-color);
    text-decoration: underline;
  }

  .editor-wrapper :global(.ProseMirror .wikilink) {
    color: var(--accent-color);
    cursor: pointer;
    text-decoration: underline;
    text-decoration-style: dashed;
  }

  .editor-wrapper :global(.ProseMirror .wikilink:hover) {
    color: var(--accent-hover);
  }

  .placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: var(--text-tertiary);
    text-align: center;
  }
</style>
