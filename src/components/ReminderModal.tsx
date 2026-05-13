import { useState } from 'react';
import type { Reminder } from '../types';

interface Props {
  current: Reminder | null;
  onSave: (reminder: Reminder | null) => void;
  onClose: () => void;
}

export function ReminderModal({ current, onSave, onClose }: Props) {
  const [time, setTime] = useState(current?.time ?? '');
  const [repeat, setRepeat] = useState<Reminder['repeat']>(current?.repeat ?? 'none');

  const handleSave = () => {
    if (!time) {
      onSave(null);
    } else {
      onSave({ time, repeat });
    }
  };

  const handleRemove = () => {
    onSave(null);
  };

  return (
    <>
      <div
        onClick={onClose}
        style={{
          position: 'fixed',
          inset: 0,
          background: 'rgba(0,0,0,0.3)',
          zIndex: 99,
        }}
      />
      <div
        style={{
          position: 'absolute',
          top: '50%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          background: '#2A2C4A',
          borderRadius: 12,
          padding: 20,
          width: 240,
          boxShadow: '0 16px 48px rgba(0,0,0,0.4)',
          zIndex: 100,
          color: '#E8E9F3',
          fontFamily: "'DM Sans', sans-serif",
        }}
      >
        <h4 style={{ margin: '0 0 12px', fontSize: 14, fontWeight: 600 }}>
          设置提醒
        </h4>

        <label style={{ fontSize: 12, color: '#9496B0', display: 'block', marginBottom: 4 }}>
          时间
        </label>
        <input
          type="datetime-local"
          value={time}
          onChange={(e) => setTime(e.target.value)}
          style={{
            width: '100%',
            padding: '6px 8px',
            background: 'rgba(255,255,255,0.06)',
            border: '1px solid rgba(255,255,255,0.1)',
            borderRadius: 6,
            color: '#E8E9F3',
            fontSize: 13,
            outline: 'none',
            marginBottom: 10,
            boxSizing: 'border-box',
          }}
        />

        <label style={{ fontSize: 12, color: '#9496B0', display: 'block', marginBottom: 4 }}>
          重复
        </label>
        <select
          value={repeat}
          onChange={(e) => setRepeat(e.target.value as Reminder['repeat'])}
          style={{
            width: '100%',
            padding: '6px 8px',
            background: 'rgba(255,255,255,0.06)',
            border: '1px solid rgba(255,255,255,0.1)',
            borderRadius: 6,
            color: '#E8E9F3',
            fontSize: 13,
            outline: 'none',
            marginBottom: 14,
            boxSizing: 'border-box',
          }}
        >
          <option value="none">不重复</option>
          <option value="daily">每天</option>
          <option value="weekly">每周</option>
          <option value="weekday">工作日</option>
        </select>

        <div style={{ display: 'flex', gap: 6, justifyContent: 'flex-end' }}>
          {current && (
            <button onClick={handleRemove} style={btnStyle('#FF6B6B', 'transparent')}>
              移除
            </button>
          )}
          <button onClick={onClose} style={btnStyle('#9496B0', 'rgba(255,255,255,0.06)')}>
            取消
          </button>
          <button onClick={handleSave} style={btnStyle('#1A1B2E', '#F0D800')}>
            保存
          </button>
        </div>
      </div>
    </>
  );
}

function btnStyle(color: string, bg: string): React.CSSProperties {
  return {
    padding: '6px 12px',
    borderRadius: 6,
    border: 'none',
    fontSize: 12,
    fontFamily: "'DM Sans', sans-serif",
    cursor: 'pointer',
    color,
    background: bg,
    fontWeight: 600,
  };
}
