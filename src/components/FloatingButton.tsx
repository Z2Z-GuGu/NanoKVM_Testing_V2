import { Plus } from 'lucide-react';

interface FloatingButtonProps {
  onClick: () => void;
  isDark: boolean;
}

export function FloatingButton({ onClick, isDark }: FloatingButtonProps) {
  return (
    <button
      onClick={onClick}
      className={`fixed bottom-8 right-8 w-14 h-14 rounded-full shadow-lg transition-all duration-200 flex items-center justify-center z-30 ${
        isDark
          ? 'bg-neutral-700 hover:bg-neutral-600 text-neutral-200'
          : 'bg-neutral-800 hover:bg-neutral-700 text-white'
      }`}
    >
      <Plus className="w-6 h-6" />
    </button>
  );
}