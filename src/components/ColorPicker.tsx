import { NOTE_COLORS } from '../types';
import type { NoteColor } from '../types';

interface Props {
  current: NoteColor;
  onSelect: (color: NoteColor) => void;
  onClose: () => void;
}

export function ColorPicker({ current, onSelect, onClose }: Props) {
  return (
    <>
      {/* Invisible backdrop to close */}
      <div
        onClick={onClose}
        style={{
          position: 'fixed',
          inset: 0,
          zIndex: 19,
        }}
      />
      <div
        style={{
          position: 'absolute',
          top: 40,
          right: 50,
          display: 'flex',
          gap: 6,
          background: 'rgba(255,255,255,0.95)',
          borderRadius: 8,
          padding: '6px 8px',
          boxShadow: '0 4px 16px rgba(0,0,0,0.15)',
          zIndex: 20,
        }}
      >
        {NOTE_COLORS.map((c) => (
          <div
            key={c.key}
            onClick={() => onSelect(c.key)}
            style={{
              width: 20,
              height: 20,
              borderRadius: '50%',
              background: c.bg,
              border: c.key === current ? `2px solid ${c.dark}` : '2px solid rgba(0,0,0,0.1)',
              cursor: 'pointer',
              transition: 'transform 0.15s',
              transform: c.key === current ? 'scale(1.2)' : 'scale(1)',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.transform = 'scale(1.3)';
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.transform = c.key === current ? 'scale(1.2)' : 'scale(1)';
            }}
          />
        ))}
      </div>
    </>
  );
}
