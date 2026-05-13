import { useEffect, useRef, useCallback, useState } from 'react';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import DOMPurify from 'dompurify';
import { useNoteStore } from '../store/noteStore';
import { getColorConfig } from '../types';
import { ColorPicker } from './ColorPicker';
import { ReminderModal } from './ReminderModal';

// Opacity settings
const OPACITY_FOCUSED = 1.0;
const OPACITY_HOVERED = 0.9;
const OPACITY_BLURRED = 0.3;
const OPACITY_TRANSITION = 'opacity 0.3s ease';

export function StickyNote({ noteId }: { noteId: string }) {
  const {
    note,
    isFocused,
    isHovered,
    loadNote,
    setTitle,
    setContent,
    setColor,
    toggleCollapse,
    togglePin,
    deleteNote,
    saveNote,
    setFocused,
    setHovered,
  } = useNoteStore();

  const [showColorPicker, setShowColorPicker] = useState(false);
  const [showReminder, setShowReminder] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const deleteTimer = useRef<ReturnType<typeof setTimeout>>(undefined);
  const contentRef = useRef<HTMLDivElement>(null);
  const saveTimer = useRef<ReturnType<typeof setTimeout>>(undefined);

  // Load note data
  useEffect(() => {
    loadNote(noteId);
  }, [noteId, loadNote]);

  // Listen for window focus/blur events
  useEffect(() => {
    const appWindow = getCurrentWebviewWindow();

    const unlistenFocus = appWindow.onFocusChanged(({ payload }) => {
      setFocused(payload);
    });

    return () => {
      unlistenFocus.then((fn) => fn());
    };
  }, [setFocused]);

  // Track window position changes
  useEffect(() => {
    const appWindow = getCurrentWebviewWindow();

    const unlistenMoved = appWindow.onMoved(({ payload }) => {
      useNoteStore.getState().updatePosition(payload.x, payload.y);
    });

    const unlistenResized = appWindow.onResized(({ payload }) => {
      useNoteStore.getState().updateSize(payload.width, payload.height);
    });

    return () => {
      unlistenMoved.then((fn) => fn());
      unlistenResized.then((fn) => fn());
    };
  }, []);

  // Debounced content save
  const handleContentChange = useCallback(() => {
    if (contentRef.current) {
      const html = contentRef.current.innerHTML;
      setContent(html);
      clearTimeout(saveTimer.current);
      saveTimer.current = setTimeout(() => saveNote(), 500);
    }
  }, [setContent, saveNote]);

  // Compute opacity
  const computedOpacity = isFocused
    ? OPACITY_FOCUSED
    : isHovered
      ? OPACITY_HOVERED
      : OPACITY_BLURRED;

  if (!note) {
    return null;
  }

  const colorConfig = getColorConfig(note.color);

  return (
    <div
      className="sticky-note"
      style={{
        background: colorConfig.bg,
        color: colorConfig.text,
        opacity: computedOpacity,
        transition: OPACITY_TRANSITION,
        width: '100%',
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        borderRadius: '3px',
        overflow: 'hidden',
        fontFamily: "'Caveat', 'Segoe UI', cursive",
        position: 'relative',
      }}
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
    >
      {/* Tape decoration */}
      <div
        style={{
          position: 'absolute',
          top: -4,
          left: '50%',
          transform: 'translateX(-50%)',
          width: 60,
          height: 16,
          background: 'rgba(255,255,255,0.3)',
          borderRadius: 2,
          zIndex: 10,
        }}
      />

      {/* Header — drag region */}
      <div
        data-tauri-drag-region
        style={{
          display: 'flex',
          alignItems: 'center',
          padding: '14px 10px 4px',
          gap: 4,
          cursor: 'grab',
          userSelect: 'none',
          zIndex: 5,
        }}
      >
        {/* Grip dots */}
        <div style={{ display: 'flex', gap: 2, marginRight: 4, opacity: 0.3 }}>
          <span style={dotStyle(colorConfig.text)} />
          <span style={dotStyle(colorConfig.text)} />
          <span style={dotStyle(colorConfig.text)} />
        </div>

        {/* Title */}
        <input
          value={note.title}
          onChange={(e) => setTitle(e.target.value)}
          spellCheck={false}
          style={{
            flex: 1,
            border: 'none',
            background: 'transparent',
            color: 'inherit',
            fontFamily: "'Caveat', cursive",
            fontSize: 18,
            fontWeight: 700,
            outline: 'none',
            cursor: 'text',
          }}
        />

        {/* Action buttons */}
        <NoteButton
          title="颜色"
          onClick={() => setShowColorPicker(!showColorPicker)}
          textColor={colorConfig.text}
        >
          <circle cx="12" cy="12" r="8" fill={colorConfig.dark} stroke={colorConfig.text} strokeWidth="1" />
        </NoteButton>

        <NoteButton
          title={note.pinned ? '取消置顶' : '置顶'}
          onClick={togglePin}
          textColor={colorConfig.text}
        >
          <path
            d="M16 12V4h1V2H7v2h1v8l-2 2v2h5.2v6h1.6v-6H18v-2l-2-2z"
            fill={colorConfig.text}
            opacity={note.pinned ? 1 : 0.4}
          />
        </NoteButton>

        <NoteButton
          title={note.collapsed ? '展开' : '收起'}
          onClick={toggleCollapse}
          textColor={colorConfig.text}
        >
          <path
            d={note.collapsed ? 'M7 14l5-5 5 5z' : 'M7 10l5 5 5-5z'}
            fill={colorConfig.text}
          />
        </NoteButton>

        <NoteButton
          title={confirmDelete ? '再次点击确认删除' : '删除'}
          onClick={() => {
            if (confirmDelete) {
              clearTimeout(deleteTimer.current);
              deleteNote();
            } else {
              setConfirmDelete(true);
              deleteTimer.current = setTimeout(() => setConfirmDelete(false), 2000);
            }
          }}
          textColor={confirmDelete ? '#FF4444' : colorConfig.text}
        >
          <path
            d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"
            fill={confirmDelete ? '#FF4444' : colorConfig.text}
          />
        </NoteButton>
      </div>

      {/* Color picker dropdown */}
      {showColorPicker && (
        <ColorPicker
          current={note.color}
          onSelect={(c) => {
            setColor(c);
            setShowColorPicker(false);
          }}
          onClose={() => setShowColorPicker(false)}
        />
      )}

      {/* Body */}
      {!note.collapsed && (
        <div
          ref={contentRef}
          contentEditable
          suppressContentEditableWarning
          onInput={handleContentChange}
          spellCheck={false}
          dangerouslySetInnerHTML={{ __html: DOMPurify.sanitize(note.content) }}
          style={{
            flex: 1,
            padding: '4px 14px 8px',
            fontFamily: "'Caveat', cursive",
            fontSize: 16,
            lineHeight: 1.6,
            outline: 'none',
            overflowY: 'auto',
            cursor: 'text',
            minHeight: 60,
          }}
        />
      )}

      {/* Footer */}
      {!note.collapsed && (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            padding: '4px 12px 8px',
            gap: 6,
            opacity: isHovered || isFocused ? 0.7 : 0,
            transition: 'opacity 0.2s',
            fontSize: 11,
          }}
        >
          <button
            onClick={() => setShowReminder(true)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: 4,
              background: 'rgba(0,0,0,0.08)',
              border: 'none',
              borderRadius: 10,
              padding: '2px 8px',
              color: 'inherit',
              cursor: 'pointer',
              fontFamily: "'DM Sans', sans-serif",
              fontSize: 11,
            }}
          >
            <svg width="12" height="12" viewBox="0 0 24 24">
              <path
                d="M12 22c1.1 0 2-.9 2-2h-4c0 1.1.89 2 2 2zm6-6v-5c0-3.07-1.64-5.64-4.5-6.32V4c0-.83-.67-1.5-1.5-1.5s-1.5.67-1.5 1.5v.68C7.63 5.36 6 7.92 6 11v5l-2 2v1h16v-1l-2-2z"
                fill="currentColor"
              />
            </svg>
            {note.reminder ? note.reminder.time : '提醒'}
          </button>
        </div>
      )}

      {/* Reminder modal */}
      {showReminder && (
        <ReminderModal
          current={note.reminder}
          onSave={(r) => {
            useNoteStore.getState().setReminder(r);
            setShowReminder(false);
          }}
          onClose={() => setShowReminder(false)}
        />
      )}
    </div>
  );
}

// ─── Sub-components ───

function dotStyle(color: string): React.CSSProperties {
  return {
    width: 3,
    height: 3,
    borderRadius: '50%',
    background: color,
    display: 'inline-block',
  };
}

function NoteButton({
  title,
  onClick,
  textColor,
  children,
}: {
  title: string;
  onClick: () => void;
  textColor: string;
  children: React.ReactNode;
}) {
  return (
    <button
      title={title}
      onClick={onClick}
      style={{
        width: 24,
        height: 24,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        border: 'none',
        background: 'transparent',
        borderRadius: 4,
        cursor: 'pointer',
        opacity: 0.5,
        transition: 'opacity 0.15s, background 0.15s',
        color: textColor,
        flexShrink: 0,
      }}
      onMouseEnter={(e) => {
        e.currentTarget.style.opacity = '1';
        e.currentTarget.style.background = 'rgba(0,0,0,0.08)';
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.opacity = '0.5';
        e.currentTarget.style.background = 'transparent';
      }}
    >
      <svg width="14" height="14" viewBox="0 0 24 24">
        {children}
      </svg>
    </button>
  );
}
