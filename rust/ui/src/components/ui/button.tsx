interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'default' | 'outline';
}

export function Button({ variant = 'outline', className = '', children, ...props }: ButtonProps) {
  const baseStyle = 'transition-all duration-200';

  const variants = {
    default: 'bg-white text-black hover:bg-white/90',
    outline: 'bg-transparent border border-white/20 text-white hover:bg-white/10 hover:border-white/30',
  };

  const buttonStyles = `${baseStyle} ${variants[variant]} ${className}`;

  return (
    <button
      className={buttonStyles}
      {...props}
    >
      {children}
    </button>
  );
}