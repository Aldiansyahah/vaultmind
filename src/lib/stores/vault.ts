import { writable } from "svelte/store";

export interface VaultEntry {
  name: string;
  path: string;
  is_directory: boolean;
  children?: VaultEntry[];
}

export const vaultPath = writable<string | null>(null);
export const vaultEntries = writable<VaultEntry[]>([]);
export const selectedNotePath = writable<string | null>(null);
export const noteContent = writable<string>("");
export const isLoading = writable(false);
export const error = writable<string | null>(null);
