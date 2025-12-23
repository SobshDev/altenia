import type { LucideIcon } from 'lucide-react';

interface SectionCardProps {
  icon: LucideIcon;
  title: string;
  description?: string;
  variant?: 'default' | 'destructive';
  staggerDelay?: number;
  children: React.ReactNode;
}

export function SectionCard({
  icon: Icon,
  title,
  description,
  variant = 'default',
  staggerDelay = 0,
  children,
}: SectionCardProps) {
  const isDestructive = variant === 'destructive';

  return (
    <div
      className={`rounded-2xl p-6 animate-fade-in-up ${
        isDestructive
          ? 'bg-destructive/5 border border-destructive/20'
          : 'glass-card glow'
      }`}
      style={{ '--stagger': `${staggerDelay}ms` } as React.CSSProperties}
    >
      <div className="flex items-start gap-4">
        <div
          className={`p-3 rounded-xl ${
            isDestructive
              ? 'bg-destructive/10 text-destructive'
              : 'bg-primary/10 text-primary'
          }`}
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
