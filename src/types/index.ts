export interface Reminder {
  time: string;
  repeat: 'none' | 'daily' | 'weekly' | 'weekday';
}

export interface Note {
  id: string;
  title: string;
  content: string;
  color: NoteColor;
  x: number;
  y: number;
  width: number;
  height: number;
  pinned: boolean;
  collapsed: boolean;
  opacity: number;
  reminder: Reminder | null;
  created_at: string;
  updated_at: string;
}

export type NoteColor = 'yellow' | 'pink' | 'blue' | 'green' | 'purple' | 'orange';

export const NOTE_COLORS: { key: NoteColor; bg: string; text: string; dark: string }[] = [
  { key: 'yellow', bg: '#FFF3B0', text: '#4A4000', dark: '#F0D800' },
  { key: 'pink', bg: '#FFD6E0', text: '#6B2040', dark: '#FF6B9D' },
  { key: 'blue', bg: '#C7ECFF', text: '#1A4060', dark: '#3B9FD8' },
  { key: 'green', bg: '#C8F7C5', text: '#1A4020', dark: '#4CAF50' },
  { key: 'purple', bg: '#E8D5F5', text: '#3A1A50', dark: '#9B59B6' },
  { key: 'orange', bg: '#FFE0C2', text: '#5A3010', dark: '#FF8C42' },
];

export function getColorConfig(color: NoteColor) {
  return NOTE_COLORS.find((c) => c.key === color) ?? NOTE_COLORS[0];
}
