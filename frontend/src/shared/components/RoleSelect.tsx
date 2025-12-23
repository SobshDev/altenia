import { useState, useRef, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { ChevronDown, Shield, User, Check } from 'lucide-react';

type Role = 'admin' | 'member';

interface RoleOption {
  value: Role;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
  description: string;
}

const roleOptions: RoleOption[] = [
  {
    value: 'admin',
    label: 'Admin',
    icon: Shield,
    description: 'Can manage members',
  },
  {
    value: 'member',
    label: 'Member',
    icon: User,
    description: 'Can view and edit',
  },
];

interface RoleSelectProps {
  value: Role;
  onChange: (value: Role) => void;
  showAdminOption?: boolean;
  size?: 'sm' | 'md';
}

export function RoleSelect({
  value,
  onChange,
  showAdminOption = true,
  size = 'md',
}: RoleSelectProps) {
  const [isOpen, setIsOpen] = useState(false);
  const buttonRef = useRef<HTMLButtonElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const options = showAdminOption
    ? roleOptions
    : roleOptions.filter((o) => o.value !== 'admin');

  const selectedOption = roleOptions.find((o) => o.value === value) || roleOptions[1];
  const SelectedIcon = selectedOption.icon;

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node) &&
        buttonRef.current &&
        !buttonRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
      }
    }

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => document.removeEventListener('mousedown', handleClickOutside);
    }
  }, [isOpen]);

  const getDropdownPosition = () => {
    if (!buttonRef.current) return { top: 0, left: 0, width: 200 };
    const rect = buttonRef.current.getBoundingClientRect();
    return {
      top: rect.bottom + 4,
      left: rect.right - 200,
      width: 200,
    };
  };

  const handleSelect = (role: Role) => {
    onChange(role);
    setIsOpen(false);
  };

  const sizeClasses = size === 'sm'
    ? 'px-2 py-1 text-xs gap-1.5'
    : 'px-3 py-2 text-sm gap-2';

  const iconSize = size === 'sm' ? 'w-3.5 h-3.5' : 'w-4 h-4';

  return (
    <>
      <button
        ref={buttonRef}
        type="button"
        onClick={() => setIsOpen(!isOpen)}
        className={`flex items-center rounded-lg bg-surface border border-border text-foreground hover:bg-surface-alt transition-colors ${sizeClasses}`}
      >
        <SelectedIcon className={`${iconSize} text-foreground-muted`} />
        <span>{selectedOption.label}</span>
        <ChevronDown
          className={`${iconSize} text-foreground-muted transition-transform ${
            isOpen ? 'rotate-180' : ''
          }`}
        />
      </button>

      {isOpen &&
        createPortal(
          <div
            ref={dropdownRef}
            className="fixed z-50 bg-surface border border-border rounded-lg shadow-lg overflow-hidden animate-fade-in-up"
            style={{
              top: getDropdownPosition().top,
              left: getDropdownPosition().left,
              width: getDropdownPosition().width,
            }}
          >
            <div className="py-1">
              {options.map((option) => {
                const Icon = option.icon;
                const isSelected = option.value === value;

                return (
                  <button
                    key={option.value}
                    type="button"
                    onClick={() => handleSelect(option.value)}
                    className={`w-full flex items-center gap-3 px-3 py-2 transition-colors ${
                      isSelected
                        ? 'bg-primary/10 text-primary'
                        : 'text-foreground hover:bg-surface-alt'
                    }`}
                  >
                    <div
                      className={`p-1.5 rounded-md ${
                        isSelected ? 'bg-primary/20' : 'bg-surface-alt'
                      }`}
                    >
                      <Icon className="w-4 h-4" />
                    </div>
                    <div className="flex-1 text-left">
                      <p className="text-sm font-medium">{option.label}</p>
                      <p className="text-xs text-foreground-muted">
                        {option.description}
                      </p>
                    </div>
                    {isSelected && <Check className="w-4 h-4 flex-shrink-0" />}
                  </button>
                );
              })}
            </div>
          </div>,
          document.body
        )}
    </>
  );
}
