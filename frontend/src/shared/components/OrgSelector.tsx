import { useState, useRef, useEffect } from 'react';
import { createPortal } from 'react-dom';
import { ChevronDown, Building2, Plus, Check, Loader2 } from 'lucide-react';
import { useOrgStore } from '@/stores/orgStore';
import { Tooltip } from '@/shared/components/Tooltip';

interface OrgSelectorProps {
  collapsed?: boolean;
}

export function OrgSelector({ collapsed = false }: OrgSelectorProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [newOrgName, setNewOrgName] = useState('');
  const buttonRef = useRef<HTMLButtonElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  const {
    organizations,
    currentOrg,
    isLoading,
    fetchOrganizations,
    switchOrg,
    createOrg,
  } = useOrgStore();

  useEffect(() => {
    fetchOrganizations();
  }, [fetchOrganizations]);

  useEffect(() => {
    if (isCreating && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isCreating]);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node) &&
        buttonRef.current &&
        !buttonRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false);
        setIsCreating(false);
        setNewOrgName('');
      }
    }

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      return () => document.removeEventListener('mousedown', handleClickOutside);
    }
  }, [isOpen]);

  const handleSelect = async (orgId: string) => {
    if (orgId === currentOrg?.id) {
      setIsOpen(false);
      return;
    }
    await switchOrg(orgId);
    setIsOpen(false);
  };

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newOrgName.trim()) return;

    try {
      await createOrg(newOrgName.trim());
      setNewOrgName('');
      setIsCreating(false);
      setIsOpen(false);
    } catch {
      // Error is handled by the store
    }
  };

  const getDropdownPosition = () => {
    if (!buttonRef.current) return { top: 0, left: 0 };
    const rect = buttonRef.current.getBoundingClientRect();
    return {
      top: rect.bottom + 4,
      left: rect.left,
      width: collapsed ? 240 : rect.width,
    };
  };

  const displayName = currentOrg?.is_personal
    ? 'Personal'
    : currentOrg?.name || 'Select org';

  const triggerButton = (
    <button
      ref={buttonRef}
      onClick={() => setIsOpen(!isOpen)}
      disabled={isLoading}
      className={`flex items-center rounded-lg transition-all duration-200 text-foreground-muted hover:bg-surface-alt hover:text-foreground ${
        collapsed ? 'justify-center p-2.5' : 'gap-2 px-3 py-2.5 w-full'
      }`}
    >
      {isLoading ? (
        <Loader2 className="w-5 h-5 animate-spin flex-shrink-0" />
      ) : (
        <Building2 className="w-5 h-5 flex-shrink-0" />
      )}
      {!collapsed && (
        <>
          <span className="flex-1 text-left truncate text-sm">{displayName}</span>
          <ChevronDown
            className={`w-4 h-4 flex-shrink-0 transition-transform ${
              isOpen ? 'rotate-180' : ''
            }`}
          />
        </>
      )}
    </button>
  );

  const dropdown = isOpen && (
    <div
      ref={dropdownRef}
      className="fixed z-50 bg-surface border border-border rounded-lg shadow-lg overflow-hidden animate-fade-in-up"
      style={{
        top: getDropdownPosition().top,
        left: getDropdownPosition().left,
        width: getDropdownPosition().width,
        minWidth: 200,
      }}
    >
      <div className="max-h-64 overflow-y-auto py-1">
        {organizations.map((org) => (
          <button
            key={org.id}
            onClick={() => handleSelect(org.id)}
            className={`w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors ${
              org.id === currentOrg?.id
                ? 'bg-primary/10 text-primary'
                : 'text-foreground hover:bg-surface-alt'
            }`}
          >
            <Building2 className="w-4 h-4 flex-shrink-0" />
            <span className="flex-1 text-left truncate">
              {org.is_personal ? 'Personal' : org.name}
            </span>
            {org.id === currentOrg?.id && (
              <Check className="w-4 h-4 flex-shrink-0" />
            )}
          </button>
        ))}
      </div>

      <div className="border-t border-border">
        {isCreating ? (
          <form onSubmit={handleCreate} className="p-2">
            <input
              ref={inputRef}
              type="text"
              value={newOrgName}
              onChange={(e) => setNewOrgName(e.target.value)}
              placeholder="Organization name"
              className="w-full px-3 py-2 text-sm bg-surface-alt border border-border rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50"
            />
            <div className="flex gap-2 mt-2">
              <button
                type="button"
                onClick={() => {
                  setIsCreating(false);
                  setNewOrgName('');
                }}
                className="flex-1 px-3 py-1.5 text-sm text-foreground-muted hover:text-foreground rounded-lg transition-colors"
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={!newOrgName.trim() || isLoading}
                className="flex-1 px-3 py-1.5 text-sm bg-primary text-primary-foreground rounded-lg hover:bg-primary-hover disabled:opacity-50 transition-colors"
              >
                Create
              </button>
            </div>
          </form>
        ) : (
          <button
            onClick={() => setIsCreating(true)}
            className="w-full flex items-center gap-2 px-3 py-2 text-sm text-foreground-muted hover:text-foreground hover:bg-surface-alt transition-colors"
          >
            <Plus className="w-4 h-4" />
            <span>Create organization</span>
          </button>
        )}
      </div>
    </div>
  );

  return (
    <>
      {collapsed ? (
        <Tooltip content={displayName} enabled={collapsed}>
          {triggerButton}
        </Tooltip>
      ) : (
        triggerButton
      )}
      {isOpen && createPortal(dropdown, document.body)}
    </>
  );
}
