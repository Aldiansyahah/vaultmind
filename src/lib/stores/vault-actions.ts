import { invoke } from "@tauri-apps/api/core";
import { get } from "svelte/store";
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
    const relativePath = name.endsWith(".md") ? name : `${name}.md`;
    await invoke("create_note", {
      relativePath,
      content: `# ${name.replace(/\.md$/, "")}\n`,
    });
    await loadVaultEntries();
    await openNote(relativePath);
  } catch (e) {
    error.set(`Failed to create note: ${e}`);
  }
}

export async function renameNote(oldPath: string, newName: string): Promise<void> {
  error.set(null);
  try {
    const newFileName = newName.endsWith(".md") ? newName : `${newName}.md`;
    // Preserve the directory portion of the original path
    const lastSlash = oldPath.lastIndexOf("/");
    const dir = lastSlash >= 0 ? oldPath.substring(0, lastSlash + 1) : "";
    const newPath = `${dir}${newFileName}`;
    await invoke("rename_note", { oldPath, newPath });
    await loadVaultEntries();
    if (get(selectedNotePath) === oldPath) {
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
    if (get(selectedNotePath) === path) {
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
    if (get(selectedNotePath) === oldPath) {
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
  } catch (e) {
    error.set(`Failed to save note: ${e}`);
  }
}

export async function set_vault_path(path: string): Promise<void> {
  await invoke("set_vault_path", { path });
}
