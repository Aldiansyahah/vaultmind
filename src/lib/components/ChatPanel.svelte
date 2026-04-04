<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { createEventDispatcher } from "svelte";

  const dispatch = createEventDispatcher();

  interface ChatMessage {
    role: "user" | "assistant";
    content: string;
    sources?: { path: string; title: string }[];
  }

  let messages = $state<ChatMessage[]>([]);
  let input = $state("");
  let isLoading = $state(false);
  let chatContainer: HTMLElement | null = null;

  async function sendMessage() {
    const text = input.trim();
    if (!text || isLoading) return;

    messages.push({ role: "user", content: text });
    input = "";
    isLoading = true;
    scrollToBottom();

    try {
      const result: { response: string; sources: { path: string; title: string }[] } = await invoke(
        "chat_with_agent",
        { message: text },
      );
      messages.push({
        role: "assistant",
        content: result.response,
        sources: result.sources,
      });
    } catch (e) {
      messages.push({
        role: "assistant",
        content: `Error: ${e}`,
      });
    } finally {
      isLoading = false;
      scrollToBottom();
    }
  }

  function scrollToBottom() {
    setTimeout(() => {
      if (chatContainer) {
        chatContainer.scrollTop = chatContainer.scrollHeight;
      }
    }, 50);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }

  function openSource(path: string) {
    dispatch("open-note", { path });
  }
</script>

<div class="chat-panel">
  <div class="chat-header">
    <h3>AI Assistant</h3>
    <button class="close-btn" onclick={() => dispatch("close")}>✕</button>
  </div>

  <div class="chat-messages" bind:this={chatContainer}>
    {#if messages.length === 0}
      <div class="empty-chat">
        <p>Ask me anything about your notes.</p>
        <p class="hint">Try: "What do I have about Rust?" or "Summarize my project notes"</p>
      </div>
    {/if}

    {#each messages as msg, i (i)}
      <div class="message {msg.role}">
        <div class="message-content">{msg.content}</div>
        {#if msg.sources && msg.sources.length > 0}
          <div class="sources">
            <span class="sources-label">Sources:</span>
            {#each msg.sources as source (source.path)}
              <button class="source-link" onclick={() => openSource(source.path)}>
                {source.title}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {/each}

    {#if isLoading}
      <div class="message assistant loading">
        <span class="dots">Thinking...</span>
      </div>
    {/if}
  </div>

  <div class="chat-input">
    <textarea
      bind:value={input}
      placeholder="Ask about your notes..."
      onkeydown={handleKeydown}
      rows="2"
    ></textarea>
    <button class="send-btn" onclick={sendMessage} disabled={isLoading || !input.trim()}>
      Send
    </button>
  </div>
</div>

<style>
  .chat-panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 350px;
    background: var(--bg-secondary);
    border-left: 1px solid var(--border-color);
  }

  .chat-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem;
    border-bottom: 1px solid var(--border-color);
  }

  .chat-header h3 {
    margin: 0;
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 1rem;
  }

  .chat-messages {
    flex: 1;
    overflow-y: auto;
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .empty-chat {
    text-align: center;
    color: var(--text-tertiary);
    padding: 2rem 1rem;
  }

  .empty-chat p {
    margin: 0.25rem 0;
  }

  .hint {
    font-size: 0.8rem;
    font-style: italic;
  }

  .message {
    padding: 0.6rem 0.75rem;
    border-radius: 8px;
    font-size: 0.85rem;
    line-height: 1.5;
    max-width: 90%;
    word-wrap: break-word;
  }

  .message.user {
    background: var(--accent-color);
    color: #fff;
    align-self: flex-end;
  }

  .message.assistant {
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    align-self: flex-start;
  }

  .message.loading {
    opacity: 0.7;
  }

  .dots {
    animation: pulse 1.5s infinite;
  }

  @keyframes pulse {
    0%,
    100% {
      opacity: 0.5;
    }
    50% {
      opacity: 1;
    }
  }

  .sources {
    margin-top: 0.5rem;
    padding-top: 0.4rem;
    border-top: 1px solid var(--border-color);
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    align-items: center;
  }

  .sources-label {
    font-size: 0.75rem;
    color: var(--text-tertiary);
  }

  .source-link {
    font-size: 0.75rem;
    color: var(--accent-color);
    background: none;
    border: 1px solid var(--border-color);
    border-radius: 4px;
    padding: 0.1rem 0.4rem;
    cursor: pointer;
  }

  .source-link:hover {
    background: var(--hover-bg);
  }

  .chat-input {
    display: flex;
    gap: 0.5rem;
    padding: 0.75rem;
    border-top: 1px solid var(--border-color);
  }

  .chat-input textarea {
    flex: 1;
    resize: none;
    padding: 0.5rem;
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    color: var(--text-primary);
    font-size: 0.85rem;
    font-family: inherit;
    outline: none;
  }

  .chat-input textarea:focus {
    border-color: var(--accent-color);
  }

  .send-btn {
    padding: 0.5rem 1rem;
    background: var(--accent-color);
    border: none;
    color: #fff;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
    align-self: flex-end;
  }

  .send-btn:hover:not(:disabled) {
    background: var(--accent-hover);
  }

  .send-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
