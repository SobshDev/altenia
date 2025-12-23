import { Check, Circle } from 'lucide-react';

interface PasswordStrengthProps {
  password: string;
  show: boolean;
}

interface Requirement {
  label: string;
  test: (password: string) => boolean;
}

const requirements: Requirement[] = [
  {
    label: 'At least 8 characters',
    test: (password) => password.length >= 8,
  },
  {
    label: 'One uppercase letter',
    test: (password) => /[A-Z]/.test(password),
  },
  {
    label: 'One lowercase letter',
    test: (password) => /[a-z]/.test(password),
  },
  {
    label: 'One number',
    test: (password) => /[0-9]/.test(password),
  },
  {
    label: 'One special character',
    test: (password) => /[!@#$%^&*(),.?":{}|<>_\-+=\[\]\\\/`~;']/.test(password),
  },
];

export function PasswordStrength({ password, show }: PasswordStrengthProps) {
  const metCount = requirements.filter((req) => req.test(password)).length;
  const allMet = metCount === requirements.length;

  return (
    <div
      className={`mt-3 space-y-2.5 overflow-hidden transition-all duration-300 ease-out ${
        show
          ? 'opacity-100 max-h-48 translate-y-0'
          : 'opacity-0 max-h-0 -translate-y-2'
      }`}
    >
      {/* Progress bar */}
      <div className="flex gap-1">
        {requirements.map((_, index) => (
          <div
            key={index}
            className={`h-1.5 flex-1 rounded-full transition-all duration-300 ${
              index < metCount
                ? allMet
                  ? 'bg-green-500'
                  : metCount >= 3
                    ? 'bg-amber-500'
                    : 'bg-orange-500'
                : 'bg-border'
            }`}
          />
        ))}
      </div>

      {/* Requirements list */}
      <div className="grid grid-cols-1 gap-1 pt-0.5">
        {requirements.map((req, index) => {
          const isMet = req.test(password);
          return (
            <div
              key={index}
              className={`flex items-center gap-2 text-xs transition-all duration-200 ${
                isMet ? 'text-green-600' : 'text-foreground-subtle'
              }`}
              style={{
                transitionDelay: show ? `${index * 30}ms` : '0ms',
              }}
            >
              <div
                className={`flex-shrink-0 w-4 h-4 rounded-full flex items-center justify-center transition-all duration-200 ${
                  isMet
                    ? 'bg-green-500 text-white'
                    : 'bg-transparent text-foreground-subtle'
                }`}
              >
                {isMet ? (
                  <Check className="w-2.5 h-2.5" strokeWidth={3} />
                ) : (
                  <Circle className="w-2 h-2" strokeWidth={3} />
                )}
              </div>
              <span>{req.label}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}

export function validatePassword(password: string): boolean {
  return requirements.every((req) => req.test(password));
}
