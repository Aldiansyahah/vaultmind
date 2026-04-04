import { Extension } from "@tiptap/core";
import { Plugin, PluginKey } from "@tiptap/pm/state";

export interface WikilinkSuggestionOptions {
  char: string;
  allowedPrefixes: string[] | null;
}

export const WikilinkSuggestion = Extension.create<WikilinkSuggestionOptions>({
  name: "wikilinkSuggestion",

  addOptions() {
    return {
      char: "[[",
      allowedPrefixes: null,
    };
  },

  addProseMirrorPlugins() {
    const { char } = this.options;

    return [
      new Plugin({
        key: new PluginKey("wikilinkSuggestion"),
        state: {
          init() {
            return { query: "", range: null as { from: number; to: number } | null };
          },
          apply(tr, value) {
            if (!tr.docChanged) return value;

            const { $from } = tr.selection;
            const textBefore = $from.parent.textBetween(
              Math.max(0, $from.parentOffset - char.length),
              $from.parentOffset,
              undefined,
              " ",
            );

            if (textBefore.endsWith(char)) {
              const textAfter = $from.parent.textBetween(
                $from.parentOffset,
                $from.parentOffset + 50,
                undefined,
                " ",
              );
              const closeIndex = textAfter.indexOf("]]");
              const queryEnd = closeIndex >= 0 ? closeIndex : textAfter.length;
              const query = textAfter.substring(0, queryEnd);

              return {
                query,
                range: {
                  from: $from.pos - char.length,
                  to: $from.pos + queryEnd,
                },
              };
            }

            return { query: "", range: null };
          },
        },
        props: {
          handleTextInput(view, from, to, text) {
            const { $from } = view.state.selection;
            const textBefore = $from.parent.textBetween(
              Math.max(0, $from.parentOffset - char.length + 1),
              $from.parentOffset,
              undefined,
              " ",
            );

            if (textBefore.endsWith(char[0]) && text === char[1]) {
              return false;
            }
            return false;
          },
        },
      }),
    ];
  },
});
