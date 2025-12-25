import type { LucideIcon } from 'lucide-react';

interface SectionCardProps {
  icon: LucideIcon;
  title: string;
  description?: string;
  variant?: 'default' | 'destructive';
  staggerDelay?: number;
  className?: string;
  children: React.ReactNode;
}

export function SectionCard({
  icon: Icon,
  title,
  description,
  variant = 'default',
  staggerDelay = 0,
  className = '',
  children,
}: SectionCardProps) {
  const isDestructive = variant === 'destructive';

  return (
    <div
      className={`rounded-2xl p-6 animate-fade-in-up ${
        isDestructive
          ? 'bg-destructive/5 border border-destructive/20 hover:border-destructive/30 transition-colors'
          : 'glass-card glow'
      } ${className}`}
      style={{ '--stagger': `${staggerDelay}ms` } as React.CSSProperties}
    >
      <div className="flex items-start gap-4">
        <div
          className={`p-3 rounded-xl animate-icon-pop card-icon ${
            isDestructive
              ? 'bg-destructive/10 text-destructive'
              : 'bg-primary/10 text-primary'
          }`}
          style={{ '--stagger': `${staggerDelay}ms` } as React.CSSProperties}
        >
          <Icon className="w-5 h-5" />
        </div>
        <div className="flex-1">
          <h2 className="text-lg font-semibold text-foreground">{title}</h2>
          {description && (
            <p className="mt-1 text-sm text-foreground-muted">{description}</p>
          )}
        </div>
      </div>

      <div className="mt-5">{children}</div>
    </div>
  );
}
