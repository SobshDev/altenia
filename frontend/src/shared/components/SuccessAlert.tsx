import { CheckCircle, X } from 'lucide-react';

interface SuccessAlertProps {
  message: string;
  onDismiss?: () => void;
}

export function SuccessAlert({ message, onDismiss }: SuccessAlertProps) {
  return (
    <div
      className="animate-success-enter rounded-xl border border-emerald-500/20 bg-emerald-500/5 p-4"
      role="alert"
    >
      <div className="flex items-start gap-3">
        <div className="flex-shrink-0 w-8 h-8 rounded-full bg-emerald-500/10 flex items-center justify-center">
          <CheckCircle className="w-4 h-4 text-emerald-500 animate-icon-bounce" />
        </div>
        <div className="flex-1 min-w-0 pt-0.5">
          <p className="text-sm font-medium text-foreground leading-relaxed">
            {message}
          </p>
        </div>
        {onDismiss && (
          <button
            type="button"
            onClick={onDismiss}
            className="flex-shrink-0 p-1 -m-1 rounded-lg text-foreground-subtle hover:text-foreground hover:bg-emerald-500/10 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        )}
      </div>
    </div>
  );
}
