import { useState, useRef, type ReactNode } from 'react';
import { createPortal } from 'react-dom';

interface TooltipProps {
  children: ReactNode;
  content: string;
  enabled?: boolean;
}

export function Tooltip({ children, content, enabled = true }: TooltipProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [position, setPosition] = useState({ top: 0, left: 0 });
  const triggerRef = useRef<HTMLSpanElement>(null);

  const handleMouseEnter = () => {
    if (triggerRef.current) {
      const rect = triggerRef.current.getBoundingClientRect();
      // Find the sidebar to position tooltip just outside it
      const sidebar = triggerRef.current.closest('aside');
      const sidebarRight = sidebar ? sidebar.getBoundingClientRect().right : rect.right;
      setPosition({
        top: rect.top + rect.height / 2,
        left: sidebarRight + 8,
      });
      setIsVisible(true);
    }
  };

  const handleMouseLeave = () => {
    setIsVisible(false);
  };

  if (!enabled) {
    return <>{children}</>;
  }

  return (
    <>
      <span
        ref={triggerRef}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        className="block"
      >
        {children}
      </span>
      {isVisible &&
        createPortal(
          <div
            style={{
              position: 'fixed',
              top: position.top,
              left: position.left,
              transform: 'translateY(-50%)',
              zIndex: 9999,
              backgroundColor: 'var(--surface-alt)',
              color: 'var(--foreground)',
              padding: '6px 12px',
              borderRadius: '6px',
              fontSize: '14px',
              boxShadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -2px rgba(0, 0, 0, 0.1)',
              border: '1px solid var(--border)',
              pointerEvents: 'none',
              whiteSpace: 'nowrap',
            }}
          >
            {content}
          </div>,
          document.body
        )}
    </>
  );
}
