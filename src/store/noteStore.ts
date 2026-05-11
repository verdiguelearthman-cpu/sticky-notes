import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import type { Note, NoteColor, Reminder } from '../types';

interface NoteStore {
  note: Note | null;
  isFocused: boolean;
  isHovered: boolean;

  // Actions
  loadNote: (id: string) => Promise<void>;
  setTitle: (title: string) => void;
  setContent: (content: string) => void;
  setColor: (color: NoteColor) => void;
  toggleCollapse: () => void;
  togglePin: () => void;
  setReminder: (reminder: Reminder | null) => void;
  deleteNote: () => Promise<void>;
  saveNote: () => Promise<void>;
  setFocused: (focused: boolean) => void;
  setHovered: (hovered: boolean) => void;
  updatePosition: (x: number, y: number) => Promise<void>;
  updateSize: (width: number, height: number) => Promise<void>;
}

export const useNoteStore = create<NoteStore>((set, get) => ({
  note: null,
  isFocused: true,
  isHovered: false,

  loadNote: async (id: string) => {
    const note = await invoke<Note | null>('get_note', { id });
    set({ note });
  },

  setTitle: (title: string) => {
    const note = get().note;
    if (note) {
      set({ note: { ...note, title } });
      get().saveNote();
    }
  },

  setContent: (content: string) => {
    const note = get().note;
    if (note) {
      set({ note: { ...note, content } });
      // Debounced save happens via component
    }
  },

  setColor: (color: NoteColor) => {
    const note = get().note;
    if (note) {
      set({ note: { ...note, color } });
      get().saveNote();
    }
  },

  toggleCollapse: () => {
    const note = get().note;
    if (note) {
      set({ note: { ...note, collapsed: !note.collapsed } });
      get().saveNote();
    }
  },

  togglePin: async () => {
    const note = get().note;
    if (note) {
      const newPinned = !note.pinned;
      set({ note: { ...note, pinned: newPinned } });
      // Sync always-on-top with Tauri window
      try {
        const { getCurrentWebviewWindow } = await import('@tauri-apps/api/webviewWindow');
        const win = getCurrentWebviewWindow();
        await win.setAlwaysOnTop(newPinned);
      } catch (_) {
        // ignore if window API not available
      }
      get().saveNote();
    }
  },

  setReminder: (reminder: Reminder | null) => {
    const note = get().note;
    if (note) {
      set({ note: { ...note, reminder } });
      get().saveNote();
    }
  },

  deleteNote: async () => {
    const note = get().note;
    if (note) {
      await invoke('delete_note', { id: note.id });
    }
  },

  saveNote: async () => {
    const note = get().note;
    if (note) {
      await invoke('update_note', { note });
    }
  },

  setFocused: (focused: boolean) => set({ isFocused: focused }),
  setHovered: (hovered: boolean) => set({ isHovered: hovered }),

  updatePosition: async (x: number, y: number) => {
    const note = get().note;
    if (note) {
      set({ note: { ...note, x, y } });
      await invoke('update_note_position', { id: note.id, x, y });
    }
  },

  updateSize: async (width: number, height: number) => {
    const note = get().note;
    if (note) {
      set({ note: { ...note, width, height } });
      await invoke('update_note_size', { id: note.id, width, height });
    }
  },
}));
