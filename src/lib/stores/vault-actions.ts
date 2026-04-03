import { invoke } from "@tauri-apps/api/core";
import { vaultEntries, selectedNotePath, noteContent, isLoading, error } from "$lib/stores/vault";
import type { VaultEntry } from "$lib/stores/vault";

export async function loadVaultEntries(): Promise<void> {
  isLoading.set(true);
  error.set(null);
  try {
    const entries: VaultEntry[] = await invoke("list_vault_files");
    vaultEntries.set(entries);
  } catch (e) {
    error.set(`Failed to load vault entries: ${e}`);
  } finally {
    isLoading.set(false);
  }
}

export async function createNewNote(name: string): Promise<void> {
  error.set(null);
  try {
    await invoke("create_note", {
      relativePath: name.endsWith(".md") ? name : `${name}.md`,
      content: `# ${name.replace(/\.md$/, "")}\n`,
    });
    await loadVaultEntries();
  } catch (e) {
    error.set(`Failed to create note: ${e}`);
  }
}

export async function renameNote(oldPath: string, newName: string): Promise<void> {
  error.set(null);
  try {
    const newPath = newName.endsWith(".md") ? newName : `${newName}.md`;
    await invoke("rename_note", { oldPath, newPath });
    await loadVaultEntries();
    if (selectedNotePath() === oldPath) {
      selectedNotePath.set(newPath);
    }
  } catch (e) {
    error.set(`Failed to rename note: ${e}`);
  }
}

export async function deleteNoteByPath(path: string): Promise<void> {
  error.set(null);
  try {
    await invoke("delete_note", { relativePath: path });
    await loadVaultEntries();
    if (selectedNotePath() === path) {
      selectedNotePath.set(null);
      noteContent.set("");
    }
  } catch (e) {
    error.set(`Failed to delete note: ${e}`);
  }
}

export async function moveNote(oldPath: string, newPath: string): Promise<void> {
  error.set(null);
  try {
    await invoke("move_note", { oldPath, newPath });
    await loadVaultEntries();
    if (selectedNotePath() === oldPath) {
      selectedNotePath.set(newPath);
    }
  } catch (e) {
    error.set(`Failed to move note: ${e}`);
  }
}

export async function openNote(path: string): Promise<void> {
  error.set(null);
  isLoading.set(true);
  try {
    const content: string = await invoke("read_note_content", { relativePath: path });
    selectedNotePath.set(path);
    noteContent.set(content);
  } catch (e) {
    error.set(`Failed to open note: ${e}`);
  } finally {
    isLoading.set(false);
  }
}

export async function saveNoteContent(path: string, content: string): Promise<void> {
  error.set(null);
  try {
    await invoke("write_note_content", { relativePath: path, content });
    noteContent.set(content);
  } catch (e) {
    error.set(`Failed to save note: ${e}`);
  }
}
