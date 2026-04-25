interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  className?: string
}

export function Input({ className = '', ...props }: InputProps) {
  return (
    <input
      className={`bg-background text-foreground px-3 py-2 outline-none outline-2 outline-transparent focus:outline-border border border-border rounded`}
      {...props}
    />
  )
}