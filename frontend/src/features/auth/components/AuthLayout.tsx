import { useLocation } from 'react-router';

function Logo({ className = '' }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 40 40"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
      className={className}
    >
      <rect width="40" height="40" rx="8" className="fill-primary" />
      <path
        d="M12 28L20 12L28 28H12Z"
        className="stroke-white"
        strokeWidth="2"
        strokeLinejoin="round"
        fill="none"
      />
      <circle cx="20" cy="22" r="3" className="fill-white" />
    </svg>
  );
}

function FloatingOrb({ className = '' }: { className?: string }) {
  return (
    <div
      className={`absolute rounded-full blur-3xl opacity-30 ${className}`}
    />
  );
}

interface AuthLayoutProps {
  children: React.ReactNode;
  title: string;
  subtitle: string;
  alternateAction: {
    text: string;
    linkText: string;
    href: string;
  };
  heroContent: React.ReactNode;
}

export function AuthLayout({ children, title, subtitle, alternateAction, heroContent }: AuthLayoutProps) {
  const location = useLocation();

  return (
    <div className="min-h-screen flex">
      {/* Left Panel - Branding */}
      <div className="hidden lg:flex lg:w-1/2 xl:w-[55%] relative overflow-hidden">
        {/* Animated gradient background */}
        <div className="absolute inset-0 gradient-bg" />

        {/* Grid pattern overlay */}
        <div className="absolute inset-0 grid-pattern opacity-30" />

        {/* Floating orbs */}
        <FloatingOrb className="w-96 h-96 bg-white -top-20 -left-20 animate-float" />
        <FloatingOrb className="w-64 h-64 bg-accent bottom-20 right-20 animate-float [animation-delay:2s]" />
        <FloatingOrb className="w-48 h-48 bg-primary top-1/3 right-1/4 animate-float [animation-delay:4s]" />

        {/* Content */}
        <div className="relative z-10 flex flex-col justify-between p-12 text-white">
          <div className="flex items-center gap-3">
            <Logo className="w-10 h-10" />
            <span className="text-2xl font-bold">Altenia</span>
          </div>

          {/* Animated hero content - key forces re-mount on route change */}
          <div key={location.pathname}>
            {heroContent}
          </div>

          <div className="text-sm text-white/50">
            &copy; {new Date().getFullYear()} Altenia. All rights reserved.
          </div>
        </div>
      </div>

      {/* Right Panel - Form */}
      <div className="flex-1 flex items-center justify-center p-6 sm:p-12 relative overflow-hidden">
        {/* Background decoration for mobile */}
        <div className="absolute inset-0 lg:hidden">
          <div className="absolute top-0 right-0 w-64 h-64 bg-primary/10 rounded-full blur-3xl" />
          <div className="absolute bottom-0 left-0 w-48 h-48 bg-accent/10 rounded-full blur-3xl" />
        </div>

        {/* Key forces re-mount and animation replay on route change */}
        <div key={location.pathname} className="w-full max-w-md relative z-10">
          {/* Mobile logo */}
          <div className="lg:hidden flex items-center justify-center gap-3 mb-8">
            <Logo className="w-10 h-10" />
            <span className="text-2xl font-bold text-foreground">Altenia</span>
          </div>

          {/* Form card with scale animation */}
          <div
            className="glass-card rounded-2xl p-8 sm:p-10 glow animate-fade-in-scale"
            style={{ '--stagger': '0ms' } as React.CSSProperties}
          >
            {/* Header with staggered reveal */}
            <div className="mb-8">
              <h2
                className="text-2xl font-bold text-foreground animate-fade-in-up"
                style={{ '--stagger': '100ms' } as React.CSSProperties}
              >
                {title}
              </h2>
              <p
                className="mt-2 text-foreground-muted animate-fade-in-up"
                style={{ '--stagger': '200ms' } as React.CSSProperties}
              >
                {subtitle}
              </p>
            </div>

            {/* Form with animation */}
            <div
              className="animate-fade-in-up"
              style={{ '--stagger': '300ms' } as React.CSSProperties}
            >
              {children}
            </div>

            {/* Footer link */}
            <div
              className="mt-8 pt-6 border-t border-border animate-fade-in-up"
              style={{ '--stagger': '500ms' } as React.CSSProperties}
            >
              <p className="text-center text-sm text-foreground-muted">
                {alternateAction.text}{' '}
                <a
                  href={alternateAction.href}
                  className="font-semibold text-primary hover:text-primary-hover transition-colors"
                >
                  {alternateAction.linkText}
                </a>
              </p>
            </div>
          </div>

          {/* Footer for mobile */}
          <p
            className="lg:hidden mt-8 text-center text-xs text-foreground-subtle animate-fade-in-up"
            style={{ '--stagger': '600ms' } as React.CSSProperties}
          >
            &copy; {new Date().getFullYear()} Altenia. All rights reserved.
          </p>
        </div>
      </div>
    </div>
  );
}
