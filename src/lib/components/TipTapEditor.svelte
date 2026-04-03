<script lang="ts">
  import { onMount } from "svelte";
  import { Editor } from "@tiptap/core";
  import StarterKit from "@tiptap/starter-kit";
  import Link from "@tiptap/extension-link";
  import { selectedNotePath, noteContent, isLoading, vaultEntries } from "$lib/stores/vault";
  import { saveNoteContent } from "$lib/stores/vault-actions";
  import WikilinkAutocomplete from "$lib/components/WikilinkAutocomplete.svelte";

  let editor: Editor | null = null;
  let container: HTMLElement | null = null;
  let saveTimeout: ReturnType<typeof setTimeout> | null = null;
  let isInternalUpdate = false;

  let showAutocomplete = $state(false);
  let wikilinkQuery = $state("");
  let selectedIndex = $state(0);
  let autocompletePosition = $state({ top: 0, left: 0 });

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

  function getSuggestions(query: string): string[] {
    if (!query) return getNoteTitles().slice(0, 10);
    const titles = getNoteTitles();
    return titles.filter((t) => t.toLowerCase().includes(query.toLowerCase())).slice(0, 10);
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
    wikilinkQuery = "";
  }

  function checkForWikilinkTrigger() {
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

    const match = textBefore.match(/\[\[([^\]]*)$/);
    if (match) {
      wikilinkQuery = match[1];
      const suggestions = getSuggestions(wikilinkQuery);
      if (suggestions.length > 0) {
        showAutocomplete = true;
        selectedIndex = 0;

        const coords = editor.view.coordsAtPos(pos);
        autocompletePosition = {
          top: coords.bottom + 5,
          left: coords.left,
        };
      } else {
        showAutocomplete = false;
      }
    } else {
      showAutocomplete = false;
    }
  }

  onMount(() => {
    editor = new Editor({
      element: container!,
      extensions: [
        StarterKit.configure({
          heading: {
            levels: [1, 2, 3],
          },
        }),
        Link.configure({
          openOnClick: false,
        }),
      ],
      content: $noteContent,
      onUpdate: () => {
        if (isInternalUpdate || !editor) return;
        const html = editor.getHTML();
        noteContent.set(html);
        checkForWikilinkTrigger();
        if (saveTimeout) clearTimeout(saveTimeout);
        saveTimeout = setTimeout(() => {
          const path = $selectedNotePath;
          if (path) {
            saveNoteContent(path, html);
          }
        }, 1000);
      },
      onSelectionUpdate: () => {
        checkForWikilinkTrigger();
      },
    });

    return () => {
      editor?.destroy();
      editor = null;
    };
  });

  $effect(() => {
    const content = $noteContent;
    if (editor && !editor.isDestroyed) {
      const currentContent = editor.getHTML();
      if (content !== currentContent) {
        isInternalUpdate = true;
        editor.commands.setContent(content);
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
  <WikilinkAutocomplete
    suggestions={getSuggestions(wikilinkQuery)}
    {selectedIndex}
    visible={showAutocomplete}
    position={autocompletePosition}
    on:select={(e) => handleWikilinkSelect(e.detail)}
    on:close={() => (showAutocomplete = false)}
  />
</div>

<style>
  .editor-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: #0f1419;
  }

  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #8899a6;
  }

  .editor-wrapper {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }

  .editor-wrapper .ProseMirror {
    outline: none;
    min-height: 100%;
    color: #e7e9ea;
    font-size: 0.95rem;
    line-height: 1.6;
  }

  .editor-wrapper .ProseMirror p {
    margin: 0.5rem 0;
  }

  .editor-wrapper .ProseMirror h1,
  .editor-wrapper .ProseMirror h2,
  .editor-wrapper .ProseMirror h3 {
    margin: 1rem 0 0.5rem;
    font-weight: 600;
  }

  .editor-wrapper .ProseMirror h1 {
    font-size: 1.75rem;
  }

  .editor-wrapper .ProseMirror h2 {
    font-size: 1.4rem;
  }

  .editor-wrapper .ProseMirror h3 {
    font-size: 1.15rem;
  }

  .editor-wrapper .ProseMirror code {
    background: #1a2332;
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-family: "JetBrains Mono", "Fira Code", monospace;
    font-size: 0.9em;
  }

  .editor-wrapper .ProseMirror pre {
    background: #1a2332;
    padding: 1rem;
    border-radius: 6px;
    overflow-x: auto;
  }

  .editor-wrapper .ProseMirror pre code {
    background: none;
    padding: 0;
  }

  .editor-wrapper .ProseMirror blockquote {
    border-left: 3px solid #2e86c1;
    padding-left: 1rem;
    margin: 0.5rem 0;
    color: #8899a6;
  }

  .editor-wrapper .ProseMirror ul,
  .editor-wrapper .ProseMirror ol {
    padding-left: 1.5rem;
    margin: 0.5rem 0;
  }

  .editor-wrapper .ProseMirror a {
    color: #2e86c1;
    text-decoration: underline;
  }

  .editor-wrapper .ProseMirror .wikilink {
    color: #2e86c1;
    cursor: pointer;
    text-decoration: underline;
    text-decoration-style: dashed;
  }

  .editor-wrapper .ProseMirror .wikilink:hover {
    color: #2471a3;
  }

  .placeholder {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #556677;
    text-align: center;
  }
</style>
