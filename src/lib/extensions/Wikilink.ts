import { Node, mergeAttributes } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";
import { Decoration, DecorationSet } from "@tiptap/pm/view";

export interface WikilinkOptions {
  HTMLAttributes: Record<string, unknown>;
  onOpenWikilink?: (target: string) => void;
}

declare module "@tiptap/core" {
  interface Commands<ReturnType> {
    wikilink: {
      insertWikilink: (target: string) => ReturnType;
    };
  }
}

export const Wikilink = Node.create<WikilinkOptions>({
  name: "wikilink",

  addOptions() {
    return {
      HTMLAttributes: {},
      onOpenWikilink: undefined,
    };
  },

  group: "inline",

  inline: true,

  addAttributes() {
    return {
      target: {
        default: null,
        parseHTML: (element) => element.getAttribute("data-target"),
        renderHTML: (attributes) => {
          if (!attributes.target) return {};
          return { "data-target": attributes.target };
        },
      },
    };
  },

  parseHTML() {
    return [
      {
        tag: "span[data-type='wikilink']",
      },
    ];
  },

  renderHTML({ HTMLAttributes }) {
    return [
      "span",
      mergeAttributes(
        { "data-type": "wikilink", class: "wikilink" },
        this.options.HTMLAttributes,
        HTMLAttributes,
      ),
      `[[${HTMLAttributes.target || ""}]]`,
    ];
  },

  addCommands() {
    return {
      insertWikilink:
        (target: string) =>
        ({ chain, state }) => {
          const { from } = state.selection;
          return chain()
            .insertContent(`[[${target}]]`)
            .setTextSelection({ from: from, to: from + target.length + 4 })
            .run();
        },
    };
  },

  addProseMirrorPlugins() {
    const { onOpenWikilink } = this.options;

    return [
      new Plugin({
        key: new PluginKey("wikilink"),
        props: {
          decorations: (state) => {
            const { doc } = state;
            const decorations: Decoration[] = [];
            const textContent = doc.textContent;

            let match;
            const regex = /\[\[([^\]]+)\]\]/g;
            while ((match = regex.exec(textContent)) !== null) {
              const from = match.index;
              const to = match.index + match[0].length;

              const pos = doc.resolve(from);
              if (pos.parent) {
                decorations.push(
                  Decoration.inline(from, to, {
                    class: "wikilink",
                    "data-target": match[1],
                  }),
                );
              }
            }

            return DecorationSet.create(doc, decorations);
          },
          handleClick: (view, pos, event) => {
            const target = (event.target as HTMLElement).closest(".wikilink");
            if (target && onOpenWikilink) {
              const wikilinkTarget = target.getAttribute("data-target");
              if (wikilinkTarget) {
                onOpenWikilink(wikilinkTarget);
              }
            }
            return false;
          },
        },
      }),
    ];
  },
});
