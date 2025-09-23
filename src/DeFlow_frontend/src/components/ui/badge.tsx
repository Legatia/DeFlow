import React from 'react';

interface BadgeProps {
  children: React.ReactNode;
  className?: string;
  variant?: 'default' | 'secondary' | 'destructive' | 'outline';
}

export const Badge: React.FC<BadgeProps> = ({ children, className = '', variant = 'default' }) => {
  const variants = {
    default: 'bg-blue-600 text-white',
    secondary: 'bg-gray-100 text-gray-800',
    destructive: 'bg-red-600 text-white',
    outline: 'border border-gray-300 bg-white text-gray-700'
  };

  return (
    <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${variants[variant]} ${className}`}>
      {children}
    </span>
  );
};